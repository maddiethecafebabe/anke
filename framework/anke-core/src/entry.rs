use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

pub trait Entry: Any + Debug + Send + Sync {
    fn source_url(&self) -> Option<String> {
        None
    }

    fn content_url(&self) -> Option<String> {
        self.image_url()
    }

    fn title_url(&self) -> Option<String> {
        None
    }

    fn image_url(&self) -> Option<String> {
        None
    }

    fn title(&self) -> Option<String> {
        None
    }

    fn tags(&self) -> Option<&HashSet<String>> {
        None
    }

    fn build_extra_fields(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

pub type EntryBox = Box<dyn Entry>;

#[derive(Debug)]
pub struct DefaultEntry {
    pub content_url: Option<String>,
    pub title_url: Option<String>,
    pub image_url: Option<String>,
    pub title: Option<String>,
    pub tags: Option<HashSet<String>>,
}

impl Entry for DefaultEntry {
    fn title_url(&self) -> Option<String> {
        self.title_url.clone()
    }

    fn image_url(&self) -> Option<String> {
        self.image_url.clone()
    }

    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn tags(&self) -> Option<&HashSet<String>> {
        (&self.tags).as_ref()
    }
}
