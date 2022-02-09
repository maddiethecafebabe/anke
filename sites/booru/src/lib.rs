//pub mod gelbooru;

//pub use gelbooru::{GelbooruEntry, GelbooruSieve};

#[macro_use]
extern crate async_trait;

mod factory;
pub use factory::{DanbooruFactory, GelbooruFactory};

mod gelbooru;
