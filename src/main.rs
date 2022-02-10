#[macro_use]
extern crate tracing;

pub use anke_core::*;
use dotenv;
use std::env;
use std::fs;

mod config;

mod app;
use app::App;

mod filters;
use filters::{BlacklistFilter, DiscordWebhookFilter};

use booru::{DanbooruFactory, GelbooruFactory};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup()?;

    let config = toml::from_str(
        &fs::read_to_string(env::var("ANKE_CONFIG").unwrap_or("./anke.toml".into())).unwrap(),
    )
    .unwrap();

    App::new(config)
        .register_aggregator_factory::<GelbooruFactory>()
        .register_aggregator_factory::<DanbooruFactory>()
        .register_filter_factory::<BlacklistFilter>()
        .register_filter_factory::<DiscordWebhookFilter>()
        .run()
        .await;

    Ok(())
}
