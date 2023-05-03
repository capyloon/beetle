pub mod bad_bits;
mod bytes_reader;
pub mod cli;
pub mod client;
pub mod config;
pub mod constants;
pub mod core;
mod cors;
mod error;
mod extractor;
pub mod handler_params;
pub mod handlers;
pub mod headers;
pub mod response;
mod rpc;
pub mod templates;
mod text;

pub(crate) const VERSION: &str = env!("CARGO_PKG_VERSION");
