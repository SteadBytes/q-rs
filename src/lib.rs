#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

use chrono::prelude::*;
use chrono::Duration;
use std::env;
use std::fs;
use std::io::prelude::*;
use std::sync::RwLock;

// TODO: Configurable options e.g. separator characters, line numbers
// colours e.t.c
#[derive(Debug)]
pub struct Formatter {}

impl Formatter {
    pub fn header(&self, now: &DateTime<Utc>, log_location: &LogLocation) -> String {
        format!(
            "[{} {} {}:{}]",
            now.time().format("%H:%M:%S").to_string(),
            log_location.file_path,
            log_location.func_path,
            log_location.lineno
        )
    }

    pub fn q(&self) -> String {
        format!(">")
    }
}

#[derive(Debug)]
pub struct LogLocation {
    file_path: String,
    func_path: String,
    lineno: u32,
}

#[derive(Debug)]
enum LoggerState {
    NotLogged,
    Logged((DateTime<Utc>, LogLocation)),
}

#[derive(Debug)]
pub struct Logger {
    formatter: Formatter,
    state: LoggerState,
    header_interval: Duration,
}

impl Logger {
    pub fn new() -> Logger {
        Logger {
            formatter: Formatter {},
            state: LoggerState::NotLogged,
            header_interval: Duration::seconds(2),
        }
    }

    fn header(&self, now: DateTime<Utc>, loc: &LogLocation) -> Option<String> {
        // FIXME: There is definitely a clearer way of implementing this!
        match &self.state {
            LoggerState::Logged((last_time, last_loc)) => {
                let elapsed = now.signed_duration_since(*last_time);
                if elapsed >= self.header_interval
                    || loc.file_path != last_loc.file_path
                    || loc.func_path != last_loc.func_path
                {
                    Some(self.formatter.header(&now, loc))
                } else {
                    None
                }
            }
            LoggerState::NotLogged => Some(self.formatter.header(&now, loc)),
        }
    }

    pub fn q(&mut self, file_path: &str, func_path: &str, lineno: u32) {
        let now = Utc::now();
        let loc = LogLocation {
            file_path: file_path.to_string(),
            func_path: func_path.to_string(),
            lineno,
        };

        let log_line = self.formatter.q();

        match self.header(now, &loc) {
            Some(header) => write_to_log(&format!("{}\n{}", header, log_line)),
            None => write_to_log(&log_line),
        }
        self.state = LoggerState::Logged((now, loc));
    }
}

lazy_static! {
    pub static ref LOGGER: RwLock<Logger> = RwLock::new(Logger::new());
}

fn write_to_log(s: &str) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(env::temp_dir().join("q"))
        .expect("Unable to open qpath");

    writeln!(file, "{}", s).expect("Unable to write to qpath");
}

// TODO: Should this be in the API? Initially implemented to facilitate reducing
// the header interval for integration tests but perhaps it's a useful feature?
// If so, should this take `std::time::Duration` instead of `chrono::Duration`?
pub fn set_header_interval(d: Duration) {
    LOGGER.write().unwrap().header_interval = d;
}

#[cfg(test)]
mod formatter_tests {
    use super::*;

    #[test]
    fn test_header() {
        let formatter = Formatter {};
        let loc = LogLocation {
            file_path: "src/lib.rs".to_string(),
            func_path: "q::tests::test_q".to_string(),
            lineno: 42,
        };

        assert_eq!(
            formatter.header(&Utc.ymd(2020, 6, 22).and_hms(20, 5, 32), &loc),
            "[20:05:32 src/lib.rs q::tests::test_q:42]"
        );
    }

    #[test]
    fn test_q() {
        let formatter = Formatter {};

        assert_eq!(formatter.q(), ">");
    }
}
