use anke_core::{Aggregator, AggregatorFactory, EntryBox, State};
use serde::Deserialize;

use crate::gelbooru::GelbooruAggregator;

fn _produce_16() -> isize {
    16
}
fn _produce_n_1() -> isize {
    -1
}

#[derive(Debug, Deserialize)]
pub struct GelbooruConfig {
    pub(crate) tags: Vec<String>,

    #[serde(default = "_produce_16")]
    pub(crate) fresh_poll_limit: isize,

    #[serde(default = "_produce_n_1")]
    pub(crate) poll_limit: isize,
}

pub struct GelbooruFactory;

impl AggregatorFactory for GelbooruFactory {
    type Config = GelbooruConfig;

    const NAME: &'static str = "gelbooru";

    fn build_aggregators(
        config: GelbooruConfig,
        state: &State,
    ) -> Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>> {

        config
            .tags
            .into_iter()
            .map(|t| GelbooruAggregator::new(t, state, config.fresh_poll_limit, config.poll_limit))
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct DanbooruConfig {
    pub(crate) tags: Vec<String>,
}

pub struct DanbooruFactory;

impl AggregatorFactory for DanbooruFactory {
    type Config = DanbooruConfig;

    const NAME: &'static str = "danbooru";

    fn build_aggregators(
        config: DanbooruConfig,
        _state: &State,
    ) -> Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>> {
        for _tag in config.tags {}

        Vec::new()
    }
}
