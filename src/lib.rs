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

#[derive(Debug, PartialEq, Clone)]
pub struct LogLocation {
    file_path: String,
    func_path: String,
    lineno: u32,
}

#[derive(Debug, PartialEq)]
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
mod logger_tests {
    use super::*;

    #[test]
    fn test_header_returns_some_if_state_not_logged() {
        let logger = Logger::new();
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };

        // Sanity check
        assert_eq!(logger.state, LoggerState::NotLogged);

        assert_eq!(
            logger.header(now, &loc),
            Some(String::from("[20:05:32 src/lib.rs crate::foo::bar:42]"))
        );
    }

    #[test]
    fn test_header_returns_some_if_state_function_differs() {
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let prev_loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };
        let logger = {
            let mut logger = Logger::new();
            logger.state = LoggerState::Logged((now, prev_loc));
            logger
        };
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::baz"),
            lineno: 42,
        };

        assert_eq!(
            logger.header(now, &loc),
            Some(String::from("[20:05:32 src/lib.rs crate::foo::baz:42]"))
        );
    }

    #[test]
    fn test_header_returns_some_if_state_module_differs() {
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let prev_loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };
        let logger = {
            let mut logger = Logger::new();
            logger.state = LoggerState::Logged((now, prev_loc));
            logger
        };
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::baz::bat"),
            lineno: 42,
        };

        assert_eq!(
            logger.header(now, &loc),
            Some(String::from("[20:05:32 src/lib.rs crate::baz::bat:42]"))
        );
    }

    #[test]
    fn test_header_returns_some_if_header_interval_elapsed() {
        let prev_time = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let header_interval = Duration::seconds(2);
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };
        let logger = {
            let mut logger = Logger::new();
            logger.state = LoggerState::Logged((prev_time, loc.clone()));
            logger.header_interval = header_interval;
            logger
        };

        // 3 seconds after `prev_time`
        // > `logger.header_interval` -> should return `Some`
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 35);

        assert_eq!(
            logger.header(now, &loc),
            Some(String::from("[20:05:35 src/lib.rs crate::foo::bar:42]"))
        );
    }

    #[test]
    fn test_header_returns_none_if_header_interval_not_elapsed_and_state_module_and_function_same()
    {
        let prev_time = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let header_interval = Duration::seconds(2);
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };
        let logger = {
            let mut logger = Logger::new();
            logger.state = LoggerState::Logged((prev_time, loc.clone()));
            logger.header_interval = header_interval;
            logger
        };

        // 1 second after `prev_time`
        // < `logger.header_interval` -> doesn't trigger header output
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 33);

        assert_eq!(
            logger.header(now, &loc), // module/function same
            None,
        );
    }
}

#[cfg(test)]
mod formatter_tests {
    use super::*;

    #[test]
    fn test_header() {
        let formatter = Formatter {};
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("q::tests::test_q"),
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
