use anke_core::{Aggregator, EntryBox, Factory, Pipeline, State};
use toml::{self, Value};

use crate::config::Config;

pub struct App {
    config: Config,
    aggregators: Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            aggregators: Vec::new(),
        }
    }

    pub async fn run(self) -> () {
        Pipeline::new(State { })
            .set_aggregators(self.aggregators)
            .setup_and_run()
            .await
    }

    pub fn register_aggregator_factory<F: Factory>(mut self) -> Self {
        lazy_static! {
            static ref EMPTY_VALUE: Value = toml::from_str("").unwrap();
        }

        let name = <F as Factory>::name();

        if let Some(config) = self.config.sources.get(name) {
            let config: <F as Factory>::Config =
                toml::from_str(&toml::to_string(config).unwrap()).unwrap();

            for agg in <F as Factory>::build_aggregators(config) {
                self.aggregators.push(agg);
            }
        }

        self
    }
}
