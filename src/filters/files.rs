use std::path::PathBuf;

use anke_core::{async_trait, EntryBox, OutputFilter, OutputFilterFactory, reqwest};
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct FilesConfig {
    root: PathBuf
}

#[derive(Debug)]
pub struct FilesSavingFilter {
    config: FilesConfig
}

impl FilesSavingFilter {
    fn new(config: FilesConfig) -> Self {
        Self {
            config
        }
    }

    async fn save_to(&self, url: &String, name: &str) -> Result<(), Box<dyn Error>> {
        let content = reqwest::get(url).await?.bytes().await?;
        let mut bytes_reader = std::io::Cursor::new(content);
        let mut dest = std::fs::File::create(self.config.root.join(name))?;

        std::io::copy(&mut bytes_reader, &mut dest)?;

        Ok(())
    }
}

#[async_trait]
impl OutputFilter for FilesSavingFilter {
    type Item = EntryBox;

    async fn filter(&mut self, entry: EntryBox) -> Option<EntryBox> {
        if let Some(url) = &entry.content_url() {
            let name = url.split("/").last().unwrap();

            if let Err(why) = self.save_to(url, name).await {
                error!("{} while downloading {} => {:?}", why, url, self.config.root.join(name));
            }
        }

        Some(entry)
    }
}

impl OutputFilterFactory for FilesSavingFilter {
    type Config = FilesConfig;

    const NAME: &'static str = "files";

    fn build_filters(config: Self::Config) -> Vec<Box<dyn OutputFilter<Item = EntryBox>>> {
        vec![Box::new(FilesSavingFilter::new(config))]
    }
}
