#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

pub mod fmt;
pub mod logging;

use chrono::Duration;
pub use logging::LogLocation;
pub use logging::Logger;
use std::env;
use std::fs;
use std::sync::RwLock;

lazy_static! {
    pub static ref LOGGER: RwLock<Logger<fs::File>> = {
        let qpath = env::temp_dir().join("q");
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&qpath)
            .expect(&format!("Unable to open log file {:?}", qpath));
        RwLock::new(Logger::new(file))
    };
}

// TODO: Should this be in the API? Initially implemented to facilitate reducing
// the header interval for integration tests but perhaps it's a useful feature?
// If so, should this take `std::time::Duration` instead of `chrono::Duration`?
// TODO: Similar function for setting `Logger.sink`?
pub fn set_header_interval(d: Duration) {
    LOGGER.write().unwrap().header_interval = d;
}
