use anke_core::{async_trait, EntryBox, OutputFilter, OutputFilterFactory};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
pub struct BlacklistConfig {
    tags: Option<HashSet<String>>,
    names: Option<HashSet<String>>,
}

#[derive(Debug)]
pub struct BlacklistFilter {
    tags: HashSet<String>,
    names: HashSet<String>,
}

impl BlacklistFilter {
    fn swallow(&self, entry: EntryBox, tag: String) -> Option<EntryBox> {
        info!(
            "Swallowed {:?} because it contained a banned tag/name: {}",
            entry, tag
        );

        None
    }
}

#[async_trait]
impl OutputFilter for BlacklistFilter {
    type Item = EntryBox;

    async fn filter(&mut self, entry: EntryBox) -> Option<EntryBox> {
        if let Some(title) = &entry.title() {
            for banned in self.names.iter() {
                if title.contains(banned) {
                    return self.swallow(entry, banned.clone());
                }
            }
        }

        if let Some(tags) = entry.tags() {
            if let Some(banned) = self.tags.intersection(&tags).next() {
                let banned = banned.clone();
                return self.swallow(entry, banned);
            }
        }

        Some(entry)
    }
}

impl OutputFilterFactory for BlacklistFilter {
    type Config = BlacklistConfig;

    const NAME: &'static str = "blacklist";

    fn build_filters(config: Self::Config) -> Vec<Box<dyn OutputFilter<Item = EntryBox>>> {
        vec![Box::new(BlacklistFilter {
            tags: config.tags.unwrap_or_default(),
            names: config.names.unwrap_or_default(),
        })]
    }
}
