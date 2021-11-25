use super::async_trait;
use std::any::Any;
use std::fmt::Debug;

#[async_trait]
pub trait Entry: Any + Debug + Send + Sync {
    async fn content_url(&self) -> Option<String> {
        self.image_url().await
    }

    async fn title_url(&self) -> Option<String>;

    async fn image_url(&self) -> Option<String>;

    async fn title(&self) -> Option<String>;
}

pub type EntryBox = Box<dyn Entry>;
