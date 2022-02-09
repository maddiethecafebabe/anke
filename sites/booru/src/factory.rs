use anke_core::{Aggregator, EntryBox, Factory, State};
use serde::Deserialize;

use crate::gelbooru::GelbooruAggregator;

#[derive(Debug, Deserialize)]
pub struct GelbooruConfig {
    pub(crate) tags: Vec<String>,
}

pub struct GelbooruFactory {}

impl Factory for GelbooruFactory {
    type Config = GelbooruConfig;

    fn name() -> &'static str {
        "gelbooru"
    }

    fn build_aggregators(
        config: GelbooruConfig,
    ) -> Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>> {
        config
            .tags
            .into_iter()
            .map(GelbooruAggregator::new)
            .collect()
    }
}

#[derive(Debug, Deserialize)]
pub struct DanbooruConfig {
    pub(crate) tags: Vec<String>,
}

pub struct DanbooruFactory {}

impl Factory for DanbooruFactory {
    type Config = DanbooruConfig;

    fn name() -> &'static str {
        "danbooru"
    }

    fn build_aggregators(
        config: DanbooruConfig,
    ) -> Vec<Box<dyn Aggregator<Item = EntryBox, PipelineState = State>>> {
        for _tag in config.tags {}

        Vec::new()
    }
}
