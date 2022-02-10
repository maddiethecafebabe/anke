#![feature(never_type, iter_intersperse)]

pub use async_trait::async_trait;

pub use tokio;

pub use reqwest;

pub use serde_json;
pub use serde_json::Value;

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

pub mod entry;
pub use entry::{Entry, EntryBox};

pub mod url;

pub use async_bucket;

use async_aggregation_pipeline::prelude;

mod state;
pub use state::{State, TokenStorageConnection};

pub type Pipeline = prelude::Pipeline<EntryBox, State>;

mod factory;
pub use factory::{AggregatorFactory, OutputFilterFactory};

pub use toml;

pub use prelude::{Aggregator, Context, OutputFilter, PipelineResult};

pub use async_aggregation_pipeline::utils::filters;
