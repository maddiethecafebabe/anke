use anke_core::{Aggregator, Context, EntryBox, PipelineResult, State};

#[derive(Debug)]
pub struct GelbooruAggregator {
    pub(crate) tag: String,
}

impl GelbooruAggregator {
    pub(crate) fn new(tag: String) -> Box<dyn Aggregator<Item = EntryBox, PipelineState = State>> {
        Box::new(Self { tag })
    }
}

#[async_trait]
impl Aggregator for GelbooruAggregator {
    type Item = EntryBox;
    type PipelineState = State;

    async fn poll(
        &mut self,
        ctx: &mut Context<Self::Item, Self::PipelineState>,
    ) -> PipelineResult<()> {
        Ok(())
    }
}
