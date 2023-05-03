use anyhow::{anyhow, Context, Result};
use clap::Parser;
use iroh_p2p::config::{Config, CONFIG_FILE_NAME, ENV_PREFIX};
use iroh_p2p::ServerConfig;
use iroh_p2p::{cli::Args, DiskStorage, Keychain, Node};
use iroh_util::lock::ProgramLock;
use iroh_util::{iroh_config_path, make_config};
use log::{debug, error};
use tokio::task;

/// Starts daemon process
fn main() -> Result<()> {
    let mut lock = ProgramLock::new("iroh-p2p")?;
    lock.acquire_or_exit();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .max_blocking_threads(2048)
        .thread_stack_size(16 * 1024 * 1024)
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let version = option_env!("IROH_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"));
        println!("Starting iroh-p2p, version {version}");

        let args = Args::parse();

        // TODO: configurable network
        let cfg_path = iroh_config_path(CONFIG_FILE_NAME)?;
        let sources = [Some(cfg_path.as_path()), args.cfg.as_deref()];
        let network_config = make_config(
            // default
            ServerConfig::default(),
            // potential config files
            &sources,
            // env var prefix for this config
            ENV_PREFIX,
            // map of present command line arguments
            args.make_overrides_map(),
        )
        .context("invalid config")?;

        #[cfg(unix)]
        {
            match iroh_util::increase_fd_limit() {
                Ok(soft) => debug!("NOFILE limit: soft = {}", soft),
                Err(err) => error!("Error increasing NOFILE limit: {}", err),
            }
        }

        let network_config = Config::from(network_config);
        let kc = Keychain::<DiskStorage>::new(network_config.key_store_path.clone()).await?;
        let rpc_addr = network_config
            .rpc_addr()
            .ok_or_else(|| anyhow!("missing p2p rpc addr"))?;
        let mut p2p = Node::new(network_config, rpc_addr, kc).await?;

        // Start services
        let p2p_task = task::spawn(async move {
            if let Err(err) = p2p.run().await {
                error!("{:?}", err);
            }
        });

        iroh_util::block_until_sigint().await;

        // Cancel all async services
        p2p_task.abort();
        p2p_task.await.ok();

        Ok(())
    })
}
