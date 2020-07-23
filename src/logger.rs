use crate::{Formatter, LogLocation};
use chrono::prelude::*;
use chrono::Duration;
use std::fmt::Debug;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Logger<W: Write> {
    formatter: Formatter,
    pub header_interval: Duration,
    sink: W,
    prev: Option<(DateTime<Utc>, LogLocation)>,
}

impl<W: Write> Logger<W> {
    pub fn new(sink: W) -> Logger<W> {
        Logger {
            formatter: Formatter {},
            header_interval: Duration::seconds(2),
            sink,
            prev: None,
        }
    }

    /// Return a header line for `loc` if a header should be included in it's
    /// corresponding log line (see below) else `None`
    ///
    /// ## Header output semantics
    ///
    /// A log line should be preceeded by a header line IFF any of the following
    /// conditions are met:
    ///
    /// - This is this first time logging with this `Logger`.
    /// - The previous log occurred in a different module to `loc`.
    /// - The previous log occurred in a different function to `loc`.
    /// - The time duration between `now` and the previous log is greater than
    ///   or equal to `self.header_interval`.
    fn header(&self, now: DateTime<Utc>, loc: &LogLocation) -> Option<String> {
        // FIXME: There is definitely a clearer way of implementing this!
        let include_header = self.prev.as_ref().map_or(true, |(last_time, last_loc)| {
            let elapsed = now.signed_duration_since(*last_time);
            elapsed >= self.header_interval
                || loc.file_path != last_loc.file_path
                || loc.func_path != last_loc.func_path
        });
        if include_header {
            Some(self.formatter.header(&now, loc))
        } else {
            None
        }
    }

    pub fn q(&mut self, loc: LogLocation) {
        let log_line = self.formatter.q();

        self.write_log_line(loc, log_line);
    }

    pub fn q_literal<T: Debug>(&mut self, val: &T, loc: LogLocation) {
        let log_line = self.formatter.q_literal(val);

        self.write_log_line(loc, log_line);
    }

    pub fn q_expr<T: Debug>(&mut self, val: &T, expr: &str, loc: LogLocation) {
        let log_line = self.formatter.q_expr(val, expr);

        self.write_log_line(loc, log_line);
    }

    /// Write `log_line` to `self.sink` using `loc` to construct a header line
    /// if necessary (see `Logger::header` for header semantics).
    fn write_log_line<S: AsRef<str>>(&mut self, loc: LogLocation, log_line: S) {
        let log_line = log_line.as_ref();
        let now = Utc::now();

        match self.header(now, &loc) {
            Some(header) => writeln!(self.sink, "{}\n{}", header, log_line),
            None => writeln!(self.sink, "{}", log_line),
        }
        .expect("Unable to write to log");

        self.prev = Some((now, loc));
    }
}
#[cfg(test)]
mod tests {
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
