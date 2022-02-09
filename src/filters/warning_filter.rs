use anke_core::{async_trait, log, EntryBox, OutputFilter};

#[derive(Debug)]
pub struct WarningFilter;

#[async_trait]
impl OutputFilter for WarningFilter {
    type Item = EntryBox;

    async fn filter(&mut self, entry: EntryBox) -> Option<EntryBox> {
        log::warn!("Unfiltered entry came through: {:?}", entry);
        None
    }
}
