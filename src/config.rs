use std::collections::HashMap;

use serde::Deserialize;
use toml::Value;

#[derive(Deserialize, Debug)]
pub struct MainConfig {
    pub database: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub main: MainConfig,
    pub sources: HashMap<String, Value>,
    pub outputs: HashMap<String, Value>,
}
