use crate::logging::LogLocation;
use chrono::prelude::*;
use std::fmt::Debug;

// TODO: Configurable options e.g. separator characters, line numbers
// colours e.t.c
#[derive(Debug)]
pub struct Formatter {}

impl Formatter {
    pub fn header(&self, now: &DateTime<Utc>, log_location: &LogLocation) -> String {
        format!(
            "[{} {:?} {} {}:{}]",
            now.time().format("%H:%M:%S").to_string(),
            log_location.thread_id,
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
    /// use q_debug::fmt::Formatter;
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
    /// use q_debug::fmt::Formatter;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    macro_rules! ll {
        ($filep: expr, $funcp:expr, $lineno: expr) => {
            (
                LogLocation {
                    file_path: String::from($filep),
                    func_path: String::from($funcp),
                    lineno: $lineno,
                    thread_id: thread::current().id(),
                },
                thread::current().id(),
            )
        };
    }

    #[test]
    fn test_header() {
        let formatter = Formatter {};
        let (loc, tid) = ll!("src/lib.rs", "q_debug::tests::test_q", 42);

        assert_eq!(
            formatter.header(&Utc.ymd(2020, 6, 22).and_hms(20, 5, 32), &loc),
            format!("[20:05:32 {:?} src/lib.rs q_debug::tests::test_q:42]", tid)
        );
    }

    #[test]
    fn test_q() {
        let formatter = Formatter {};

        assert_eq!(formatter.q(), ">");
    }
    //
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
