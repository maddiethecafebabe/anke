use crate::EntryBox;
use crate::State;
use async_aggregation_pipeline::{aggregator::Aggregator, prelude::OutputFilter};
use serde::de::DeserializeOwned;

pub trait AggregatorFactory {
    type Config: DeserializeOwned;

    const NAME: &'static str;

    fn build_aggregators(
        config: Self::Config,
        state: &State,
    ) -> Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>>;
}

pub trait OutputFilterFactory {
    type Config: DeserializeOwned;

    const NAME: &'static str;

    fn build_filters(config: Self::Config) -> Vec<Box<dyn OutputFilter<Item = EntryBox>>>;
}
