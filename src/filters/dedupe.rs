use anke_core::{async_trait, EntryBox, OutputFilter, OutputFilterFactory};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct DedupeConfig { }

#[derive(Debug)]
pub struct DedupeFilter {
    memory: HashSet<String>,
}

impl DedupeFilter {
    pub fn new(_: DedupeConfig) -> Self {
        Self {
            memory: HashSet::with_capacity(1000)
        }
    }
}


#[async_trait]
impl OutputFilter for DedupeFilter {
    type Item = EntryBox;

    async fn filter(&mut self, entry: EntryBox) -> Option<EntryBox> {
        if let Some(url) = &entry.content_url() {
            if self.memory.contains(url) {
                info!("Dropped recently seen duplicate: {:?}", entry);
                return None;
            } else {
                self.memory.insert(url.clone());
            }
        }

        Some(entry)
    }
}

impl OutputFilterFactory for DedupeFilter {
    type Config = DedupeConfig;

    const NAME: &'static str = "dedupe";

    fn build_filters(config: Self::Config) -> Vec<Box<dyn OutputFilter<Item = EntryBox>>> {
        vec![Box::new(DedupeFilter::new(config))]
    }
}
