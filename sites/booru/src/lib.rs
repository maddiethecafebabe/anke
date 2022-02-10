//pub mod gelbooru;

//pub use gelbooru::{GelbooruEntry, GelbooruSieve};

#[macro_use]
extern crate async_trait;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate lazy_static;

mod factory;
pub use factory::{DanbooruFactory, GelbooruFactory};

mod gelbooru;
