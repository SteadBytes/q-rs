#[macro_use]
extern crate lazy_static;

#[macro_use]
mod macros;

use chrono::prelude::*;
use chrono::Duration;
use std::env;
use std::fmt::Debug;
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

    /// Returns a log line for a literal value, `val`.
    ///
    /// ```
    /// use q::Formatter;
    ///
    /// let fmt = Formatter {};
    ///
    /// assert_eq!(
    ///     fmt.q_literal(&String::from("Test message")),
    ///     String::from("> \"Test message\"")
    /// );
    ///
    /// assert_eq!(
    ///     fmt.q_literal(&Some(42)),
    ///     String::from("> Some(42)")
    /// );
    /// ```
    pub fn q_literal<T: Debug>(&self, val: &T) -> String {
        format!("> {:?}", val)
    }

    /// Returns a log line for an expression with string representation `expr`
    /// and value `val`.
    ///
    /// ```
    /// use q::Formatter;
    ///
    /// let fmt = Formatter {};
    ///
    /// assert_eq!(
    ///     fmt.q_expr(&3, &String::from("my_var")),
    ///     String::from("> my_var = 3")
    /// );
    ///
    /// assert_eq!(
    ///     fmt.q_expr(&5, &String::from("2 + 3")),
    ///     String::from("> 2 + 3 = 5")
    /// );
    /// ```
    pub fn q_expr<T: Debug>(&self, val: &T, expr: &str) -> String {
        format!("> {} = {:?}", expr, val)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct LogLocation {
    pub file_path: String,
    pub func_path: String,
    pub lineno: u32,
}

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

#[cfg(test)]
mod logger_tests {
    use super::*;

    fn testing_logger() -> Logger<Vec<u8>> {
        Logger::new(vec![])
    }

    #[test]
    fn test_header_returns_some_if_not_logged_previously() {
        let logger = testing_logger();
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };

        // Sanity check
        assert_eq!(logger.prev, None);

        assert_eq!(
            logger.header(now, &loc),
            Some(String::from("[20:05:32 src/lib.rs crate::foo::bar:42]"))
        );
    }

    #[test]
    fn test_header_returns_some_if_prev_function_differs() {
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let prev_loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };
        let logger = {
            let mut logger = testing_logger();
            logger.prev = Some((now, prev_loc));
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
    fn test_header_returns_some_if_prev_module_differs() {
        let now = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let prev_loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };
        let logger = {
            let mut logger = testing_logger();
            logger.prev = Some((now, prev_loc));
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
            let mut logger = testing_logger();
            logger.prev = Some((prev_time, loc.clone()));
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
    fn test_header_returns_none_if_header_interval_not_elapsed_and_prev_module_and_function_same() {
        let prev_time = Utc.ymd(2020, 6, 22).and_hms(20, 5, 32);
        let header_interval = Duration::seconds(2);
        let loc = LogLocation {
            file_path: String::from("src/lib.rs"),
            func_path: String::from("crate::foo::bar"),
            lineno: 42,
        };
        let logger = {
            let mut logger = testing_logger();
            logger.prev = Some((prev_time, loc.clone()));
            // logger.prev = LoggerState::Logged((prev_time, loc.clone()));
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

    #[test]
    fn test_q_literal() {
        let formatter = Formatter {};

        assert_eq!(
            formatter.q_literal(&String::from("Hello, world!")),
            "> \"Hello, world!\""
        );
        assert_eq!(formatter.q_literal(&1), "> 1");
    }

    #[test]
    fn test_q_expr() {
        let formatter = Formatter {};

        assert_eq!(
            formatter.q_expr(&String::from("Hello, world!"), &String::from("my_var")),
            "> my_var = \"Hello, world!\""
        );
        assert_eq!(
            formatter.q_expr(&Some(42), &String::from("a_function(42)")),
            "> a_function(42) = Some(42)"
        );
    }
}
