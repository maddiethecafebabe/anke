use crate::EntryBox;
use async_aggregation_pipeline::aggregator::Aggregator;
use serde::de::DeserializeOwned;
use crate::State;

pub trait Factory {
    type Config: DeserializeOwned;

    fn name() -> &'static str;

    fn build_aggregators(
        config: Self::Config,
    ) -> Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>>;
}
