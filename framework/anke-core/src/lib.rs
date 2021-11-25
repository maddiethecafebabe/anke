#![feature(never_type, iter_intersperse)]

pub use async_trait::async_trait;

pub use tokio;

pub use reqwest;

pub use serde_json;

pub use crossbeam_channel;

pub mod log {
    pub use tracing::{debug, error, info, trace, warn};
}

pub type Result<T> = std::result::Result<T, color_eyre::Report>;

pub fn setup() -> Result<()> {
    use tracing_subscriber::{filter::EnvFilter, fmt};

    color_eyre::install()?;
    fmt::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    Ok(())
}

mod filter_net;
pub use filter_net::FilterNet;

mod sieve;
pub use sieve::{AsyncSender, Sieve, SieveContext};

mod output_filter;
pub use output_filter::OutputFilter;

mod entry;
pub use entry::{Entry, EntryBox};

pub mod url;

pub mod pipeline;
pub use pipeline::Pipeline;

pub use async_bucket;
