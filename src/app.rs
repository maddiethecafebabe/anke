use anke_core::{
    Aggregator, AggregatorFactory, EntryBox, OutputFilter, OutputFilterFactory, Pipeline, State,
};

use crate::config::Config;

pub struct App {
    config: Config,
    aggregators: Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>>,
    filters: Vec<Box<dyn OutputFilter<Item = EntryBox>>>,
    state: State,
}

impl App {
    pub fn new(config: Config) -> Self {
        let state = State::new(config.main.database.clone());

        Self {
            config,
            aggregators: Vec::new(),
            filters: Vec::new(),
            state,
        }
    }

    pub async fn run(self) -> () {
        Pipeline::new(self.state)
            .set_aggregators(self.aggregators)
            .set_output_filters(self.filters)
            .setup_and_run()
            .await
    }

    pub fn register_aggregator_factory<F: AggregatorFactory>(mut self) -> Self {
        let name = <F as AggregatorFactory>::NAME;

        if let Some(config) = self.config.sources.get(name) {
            let config: <F as AggregatorFactory>::Config =
                toml::from_str(&toml::to_string(config).unwrap()).unwrap();

            for mut agg in <F as AggregatorFactory>::build_aggregators(config, &self.state) {
                agg.on_load();
                self.aggregators.push(agg);
            }
        }

        self
    }

    pub fn register_filter_factory<F: OutputFilterFactory>(mut self) -> Self {
        let name = <F as OutputFilterFactory>::NAME;

        if let Some(config) = self.config.outputs.get(name) {
            let config: <F as OutputFilterFactory>::Config =
                toml::from_str(&toml::to_string(config).unwrap()).unwrap();

            for mut f in <F as OutputFilterFactory>::build_filters(config) {
                f.on_load();
                self.filters.push(f);
            }
        }

        self
    }

    #[allow(dead_code)]
    pub fn add_output_filter(mut self, filter: impl OutputFilter<Item = EntryBox>) -> Self {
        self.filters.push(Box::new(filter));

        self
    }
}
