use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Display, Formatter},
    hash::BuildHasher,
    str::FromStr,
    sync::Arc,
    sync::Mutex,
};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use bytes::Bytes;
use cid::{multibase::Base, Cid};
use futures::future::Either;
use futures::StreamExt;
use iroh_rpc_client::Client;
use reqwest::Url;
use tracing::{debug, info, trace, warn};

use crate::{
    builder::FileBuilder,
    indexer::{Indexer, IndexerUrl},
    parse_links,
    types::{LoadedCid, Source},
    uploads::make_dir_from_file_list,
};

pub const IROH_STORE: &str = "iroh-store";

#[async_trait]
pub trait ContentLoader: Sync + Send + std::fmt::Debug + Clone + 'static {
    /// Loads the actual content of a given cid.
    async fn load_cid(&self, cid: &Cid, ctx: &LoaderContext) -> Result<LoadedCid>;
    /// Signal that the passend in session is not used anymore.
    async fn stop_session(&self, ctx: ContextId) -> Result<()>;
    /// Checks if the given cid is present in the local storage.
    async fn has_cid(&self, cid: &Cid) -> Result<bool>;
    /// Store multiple files as a single unixFS directory
    async fn store_files<T: tokio::io::AsyncRead + 'static + std::marker::Send>(
        &self,
        _entries: Vec<(T, String)>,
    ) -> Result<cid::Cid, anyhow::Error> {
        unimplemented!()
    }

    /// Store a single file
    async fn store_file<T: tokio::io::AsyncRead + 'static + std::marker::Send>(
        &self,
        _file: T,
    ) -> Result<cid::Cid, anyhow::Error> {
        unimplemented!()
    }
}

#[async_trait]
impl<T: ContentLoader> ContentLoader for Arc<T> {
    async fn load_cid(&self, cid: &Cid, ctx: &LoaderContext) -> Result<LoadedCid> {
        self.as_ref().load_cid(cid, ctx).await
    }

    async fn stop_session(&self, ctx: ContextId) -> Result<()> {
        self.as_ref().stop_session(ctx).await
    }

    async fn has_cid(&self, cid: &Cid) -> Result<bool> {
        self.as_ref().has_cid(cid).await
    }

    async fn store_files<C: tokio::io::AsyncRead + 'static + std::marker::Send>(
        &self,
        entries: Vec<(C, String)>,
    ) -> Result<cid::Cid, anyhow::Error> {
        self.as_ref().store_files(entries).await
    }

    async fn store_file<C: tokio::io::AsyncRead + 'static + std::marker::Send>(
        &self,
        file: C,
    ) -> Result<cid::Cid, anyhow::Error> {
        self.as_ref().store_file(file).await
    }
}

#[derive(Debug, Clone)]
pub struct FullLoader {
    /// RPC Client.
    client: Client,
    /// API to talk to the indexer nodes.
    indexer: Option<Indexer>,
    /// Gateway endpoints.
    http_gateways: Vec<GatewayUrl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullLoaderConfig {
    pub indexer: Option<IndexerUrl>,
    pub http_gateways: Vec<GatewayUrl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatewayUrl {
    Full(Url),
    Subdomain(String),
}

impl FromStr for GatewayUrl {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self> {
        if input.starts_with("http") || input.starts_with("https") {
            let url = input.parse()?;
            return Ok(GatewayUrl::Full(url));
        }

        Ok(GatewayUrl::Subdomain(input.to_string()))
    }
}

impl GatewayUrl {
    pub fn as_string(&self) -> String {
        match self {
            GatewayUrl::Full(url) => url.to_string(),
            GatewayUrl::Subdomain(s) => s.clone(),
        }
    }

    pub fn as_url(&self, cid: &Cid) -> Result<Url> {
        let cid_str = cid.into_v1()?.to_string_of_base(Base::Base32Lower)?;
        let url = match self {
            GatewayUrl::Full(raw) => {
                let mut url = raw.join(&cid_str).unwrap();
                url.set_query(Some("format=raw"));
                url
            }
            GatewayUrl::Subdomain(raw) => {
                format!("https://{cid_str}.ipfs.{raw}?format=raw").parse()?
            }
        };
        Ok(url)
    }
}

impl FullLoader {
    pub fn new(client: Client, config: FullLoaderConfig) -> Result<Self> {
        let indexer = config.indexer.map(Indexer::new).transpose()?;

        Ok(Self {
            client,
            indexer,
            http_gateways: config.http_gateways,
        })
    }

    async fn fetch_store(&self, cid: &Cid) -> Result<Option<LoadedCid>> {
        match self.client.try_store() {
            Ok(store) => Ok(store.get(*cid).await?.map(|data| LoadedCid {
                data,
                source: Source::Store(IROH_STORE),
            })),
            Err(err) => {
                info!("No store available: {:?}", err);
                Ok(None)
            }
        }
    }

    async fn fetch_bitswap(&self, ctx: ContextId, cid: &Cid) -> Result<Option<LoadedCid>> {
        match self.client.try_p2p() {
            Ok(p2p) => {
                let providers: HashSet<_> = if let Some(ref indexer) = self.indexer {
                    if let Ok(providers) = indexer.find_providers(*cid).await {
                        providers.into_iter().map(|p| p.id).collect()
                    } else {
                        Default::default()
                    }
                } else {
                    Default::default()
                };

                let data = p2p.fetch_bitswap(ctx.into(), *cid, providers).await?;
                Ok(Some(LoadedCid {
                    data,
                    source: Source::Bitswap,
                }))
            }
            Err(err) => {
                info!("No p2p available: {:?}", err);
                Ok(None)
            }
        }
    }

    // Try to fetch the cid from one of the configured gateways.
    async fn fetch_gateway(&self, cid: &Cid) -> Result<Option<LoadedCid>> {
        let mut last_error = Ok(None);

        for url in &self.http_gateways {
            let response = reqwest::get(url.as_url(cid)?).await?;
            if !response.status().is_success() {
                last_error = Err(anyhow!("unexpected http status"));
                continue;
            }
            let data = response.bytes().await?;
            // Make sure the content is not tampered with.
            if iroh_util::verify_hash(cid, &data) == Some(true) {
                return Ok(Some(LoadedCid {
                    data,
                    source: Source::Http(url.as_string()),
                }));
            } else {
                last_error = Err(anyhow!("invalid CID hash"));
            }
        }

        last_error
    }

    fn store_data(&self, cid: Cid, data: Bytes) {
        // trigger storage in the background
        let store = self.client.try_store();
        let p2p = self.client.try_p2p();

        tokio::spawn(async move {
            let links = tokio::task::spawn_blocking({
                let data = data.clone();
                move || parse_links(&cid, &data).unwrap_or_default()
            })
            .await
            .unwrap_or_default();

            if let Ok(store_rpc) = store {
                match store_rpc.put(cid, data.clone(), links).await {
                    Ok(_) => {
                        // Notify bitswap about new blocks
                        if let Ok(p2p) = p2p {
                            p2p.notify_new_blocks_bitswap(vec![(cid, data)]).await.ok();
                        }
                    }
                    Err(err) => {
                        warn!("failed to store {}: {:?}", cid, err);
                    }
                }
            } else {
                warn!("failed to store: missing store rpc conn");
            }
        });
    }
}

#[async_trait]
impl ContentLoader for FullLoader {
    async fn stop_session(&self, ctx: ContextId) -> Result<()> {
        self.client
            .try_p2p()?
            .stop_session_bitswap(ctx.into())
            .await?;
        Ok(())
    }

    async fn load_cid(&self, cid: &Cid, ctx: &LoaderContext) -> Result<LoadedCid> {
        trace!("{:?} loading {}", ctx.id(), cid);

        if let Some(loaded) = self.fetch_store(cid).await? {
            return Ok(loaded);
        }

        let bitswap_future = self.fetch_bitswap(ctx.id(), cid);
        let gateway_future = self.fetch_gateway(cid);

        tokio::pin!(bitswap_future);
        tokio::pin!(gateway_future);

        let res = futures::future::select(bitswap_future, gateway_future).await;
        let loaded = match res {
            Either::Left((bitswap, gateway_fut)) => {
                if let Ok(Some(loaded)) = bitswap {
                    loaded
                } else {
                    let gateway = gateway_fut.await;
                    if let Ok(Some(loaded)) = gateway {
                        loaded
                    } else {
                        let bitswap_offline = matches!(bitswap, Ok(None));
                        let gateway_offline = matches!(gateway, Ok(None));
                        if bitswap_offline && gateway_offline {
                            return Err(anyhow!("offline"));
                        }
                        return Err(anyhow!("failed to find {}", cid));
                    }
                }
            }
            Either::Right((gateway, bitswap_future)) => {
                if let Ok(Some(loaded)) = gateway {
                    loaded
                } else {
                    let bitswap = bitswap_future.await;
                    if let Ok(Some(loaded)) = bitswap {
                        loaded
                    } else {
                        let bitswap_offline = matches!(bitswap, Ok(None));
                        let gateway_offline = matches!(gateway, Ok(None));
                        if bitswap_offline && gateway_offline {
                            return Err(anyhow!("offline"));
                        }
                        return Err(anyhow!("failed to find {}", cid));
                    }
                }
            }
        };

        self.store_data(*cid, loaded.data.clone());
        Ok(loaded)
    }

    async fn has_cid(&self, cid: &Cid) -> Result<bool> {
        self.client.try_store()?.has(*cid).await
    }

    // Store a single file and returns the root cid for the stored content.
    async fn store_file<T: tokio::io::AsyncRead + 'static + std::marker::Send>(
        &self,
        content: T,
    ) -> Result<cid::Cid, anyhow::Error> {
        let store = self.client.try_store()?;

        let file_builder = FileBuilder::new()
            .content_reader(content)
            .name("_http_upload_");
        let file = file_builder.build().await?;

        let mut cids: Vec<cid::Cid> = vec![];
        let mut blocks = Box::pin(file.encode().await?);
        while let Some(block) = blocks.next().await {
            let (cid, bytes, links) = block.unwrap().into_parts();
            cids.push(cid);
            store.put(cid, bytes, links).await?;
        }

        match cids.last() {
            Some(root_cid) => {
                // This fails if Kademlia is not enabled, which can be the case
                // and it's ok.
                let _ = self.client.try_p2p()?.start_providing(&root_cid).await;
                Ok(*root_cid)
            }
            None => Err(anyhow!("no root cid!")),
        }
    }

    // Store a set of file and returns the root cid for the stored content.
    async fn store_files<T: tokio::io::AsyncRead + 'static + std::marker::Send>(
        &self,
        entries: Vec<(T, String)>,
    ) -> Result<cid::Cid, anyhow::Error> {
        let store = self.client.try_store()?;
        let mut cids: Vec<cid::Cid> = vec![];

        let directory = make_dir_from_file_list(entries).await?;

        let mut blocks = directory.encode();
        while let Some(block) = blocks.next().await {
            let (cid, bytes, links) = block.unwrap().into_parts();
            cids.push(cid);
            store.put(cid, bytes, links).await?;
        }

        match cids.last() {
            Some(root_cid) => {
                // This fails if Kademlia is not enabled, which can be the case
                // and it's ok.
                let _ = self.client.try_p2p()?.start_providing(&root_cid).await;
                Ok(*root_cid)
            }
            None => Err(anyhow!("no root cid!")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoaderContext {
    id: ContextId,
    inner: Arc<Mutex<InnerLoaderContext>>,
}

impl LoaderContext {
    pub fn from_path(id: ContextId, closer: async_channel::Sender<ContextId>) -> Self {
        trace!("new loader context: {:?}", id);
        LoaderContext {
            id,
            inner: Arc::new(Mutex::new(InnerLoaderContext { closer })),
        }
    }

    pub fn id(&self) -> ContextId {
        self.id
    }
}

impl Drop for LoaderContext {
    fn drop(&mut self) {
        let count = Arc::strong_count(&self.inner);
        debug!("session {} dropping loader context {}", self.id, count);
        if count == 1 {
            if let Err(err) = self
                .inner
                .try_lock()
                .expect("last reference, no lock")
                .closer
                .send_blocking(self.id)
            {
                warn!(
                    "failed to send session stop for session {}: {:?}",
                    self.id, err
                );
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContextId(pub u64);

impl Display for ContextId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ContextId({})", self.0)
    }
}

impl From<u64> for ContextId {
    fn from(id: u64) -> Self {
        ContextId(id)
    }
}

impl From<ContextId> for u64 {
    fn from(id: ContextId) -> Self {
        id.0
    }
}

#[derive(Debug)]
pub struct InnerLoaderContext {
    closer: async_channel::Sender<ContextId>,
}

#[async_trait]
impl<S: BuildHasher + Clone + Send + Sync + 'static> ContentLoader for HashMap<Cid, Bytes, S> {
    async fn load_cid(&self, cid: &Cid, _ctx: &LoaderContext) -> Result<LoadedCid> {
        match self.get(cid) {
            Some(b) => Ok(LoadedCid {
                data: b.clone(),
                source: Source::Bitswap,
            }),
            None => bail!("not found"),
        }
    }

    async fn stop_session(&self, _ctx: ContextId) -> Result<()> {
        // no session tracking
        Ok(())
    }

    async fn has_cid(&self, cid: &Cid) -> Result<bool> {
        Ok(self.contains_key(cid))
    }
}
