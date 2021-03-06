pub mod couch;
pub mod elastic;
pub mod rss;
pub mod server;
pub mod tasks;
pub mod util;

pub use oas_common::*;

pub struct State {
    pub db: couch::CouchDB,
    pub index: elastic::Index,
}

impl State {
    // pub fn init() -> Self {}
}
