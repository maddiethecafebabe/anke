use crate::{async_trait, EntryBox};
use std::fmt::Debug;

#[async_trait]
pub trait OutputFilter: Debug + Send + Sync {
    async fn filter(&mut self, entry: EntryBox) -> Option<EntryBox>;
}
