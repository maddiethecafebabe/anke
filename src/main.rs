#[macro_use]
extern crate lazy_static;

pub use anke_core::*;
use dotenv;
use std::fs;

mod config;

mod app;
use app::App;

mod filters;

use booru::{DanbooruFactory, GelbooruFactory};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup()?;

    let config = toml::from_str(&fs::read_to_string("./anke.toml").unwrap()).unwrap();

    App::new(config)
        .register_aggregator_factory::<GelbooruFactory>()
        .register_aggregator_factory::<DanbooruFactory>()
        .run()
        .await;

    Ok(())
}
