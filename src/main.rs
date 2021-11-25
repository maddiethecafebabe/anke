pub use anke_core::*;
pub use booru::GelbooruSieve;
use dotenv;
use serde::Deserialize;

use std::collections::HashMap;
use std::env;
use std::fs::File;

mod app;
use app::App;

mod filters;
use filters::*;

#[derive(Debug, Deserialize)]
pub struct HostsConfig {
    #[allow(dead_code)] // its read but idk
    sites: HashMap<String, Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup()?;

    let config: HostsConfig = serde_json::from_reader(
        File::open(env::args().nth(1).unwrap_or("./hosts.json".into())).unwrap(),
    )
    .unwrap();

    App::new(config)
        .map_patterns_to_sieve(["gelbooru.com"], GelbooruSieve::new())
        .add_result_filter(DiscordWebhookFilter::from_env())
        .add_result_filter(WarningFilter)
        .run_forever()
        .await
}
