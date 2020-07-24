#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate q;

use chrono;
use regex::Regex;
use std::env;
use std::fs;
use std::io::Read;
use std::sync::Mutex;
use std::thread::sleep;
use std::time;

lazy_static! {
    static ref QFILE: Mutex<fs::File> = {
        Mutex::new(
            fs::OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(env::temp_dir().join("q"))
                .unwrap(),
        )
    };
}

// TODO: Move some of these into `Logger` unit tests now that an in memory
//       buffer can be used instead of reading /tmp/q.
// TODO: Improve log line parsing/matching

macro_rules! read_log {
    ($e:expr) => {{
        let mut file = QFILE.lock().unwrap();
        let mut output = String::new();

        file.read_to_string(&mut output).unwrap();
        output.clear();

        $e;

        file.read_to_string(&mut output).unwrap();

        output.trim().to_owned()
    }};
}

lazy_static! {
    static ref HEADER_RE: Regex = Regex::new(
        r"\[(?P<time>\d{2}:\d{2}:\d{2}) (?P<thread_id>.*) tests/smoke\.rs (?P<funcp>.*):(?P<lineno>\d+)\]"
    )
    .unwrap();

    static ref LOG_LINE_RE: Regex = Regex::new(r">( (?P<msg>.*))?").unwrap();
}

fn check_header_line(header_line: &str, func_path: &str, lineno: u32) {
    let caps = HEADER_RE.captures(&header_line.trim()).unwrap();
    assert_eq!(&caps["funcp"], func_path);
    assert_eq!(caps["lineno"].parse::<u32>().unwrap(), lineno);
}

fn check_log_line(log_line: &str, msg: &str) {
    println!("{}", log_line);
    let caps = LOG_LINE_RE.captures(&log_line.trim()).unwrap();
    if msg == "" {
        assert_eq!(caps.name("msg"), None);
    } else {
        assert_eq!(&caps["msg"], msg);
    }
}

// FIXME: This test checks for a header without forcing the correct conditions
// to ensure a header is logged - essentially it relies on being run first.
#[test]
fn test_empty_call() {
    let line_before_call = line!();
    let output = read_log!(q!());

    check_header_line(&output, "smoke::test_empty_call", line_before_call + 1);
    check_log_line(&output, "");
}

#[test]
fn test_literal() {
    let output = read_log!(q!("Test message"));

    // Log output is correct
    check_log_line(&output, "\"Test message\"");

    // q! returns value
    assert_eq!(q!("Test message"), "Test message");
}

#[test]
fn test_ident() {
    let x = 1;
    let output = read_log!(q!(x));

    // Log output is correct
    check_log_line(&output, "x = 1");

    // q! returns value
    assert_eq!(q!(x), x);
}

#[test]
fn test_expr() {
    fn add_two(x: i32) -> i32 {
        x + 2
    }

    let output = read_log!(q!(add_two(2)));

    // Log output is correct
    check_log_line(&output, "add_two(2) = 4");

    // q! returns expression value
    assert_eq!(q!(add_two(2)), 4);
}

// FIXME: This test changes the header interval for the *global* logger and
// could therefore affect other tests. Is there another way to reduce the
// interval just for this one test?
#[test]
fn test_header_interval() {
    // Set long interval to ensure no header between first two calls
    q::set_header_interval(chrono::Duration::seconds(2));

    let output_1 = read_log!(q!()); // Header
    let output_2 = read_log!(q!()); // No header

    // Set short interval to trigger header output w/out making the test too slow
    q::set_header_interval(chrono::Duration::milliseconds(200));

    // Sleep for > header interval and call again to trigger header output
    sleep(time::Duration::from_millis(250));

    let output_3 = read_log!(q!()); // Header
    let output_4 = read_log!(q!()); // No header

    assert!(HEADER_RE.is_match(&output_1));
    assert!(!HEADER_RE.is_match(&output_2));
    assert!(HEADER_RE.is_match(&output_3));
    assert!(!HEADER_RE.is_match(&output_4));
}

#[test]
fn test_header_on_function_or_module_change() {
    // Set long interval to ensure header is only triggered by function/module
    // change
    q::set_header_interval(chrono::Duration::seconds(2));

    let output_1 = read_log!(q!()); // Header
    let output_2 = read_log!(q!()); // No header

    fn foo() -> String {
        read_log!(q!())
    }

    let output_3 = foo(); // Header
    let output_4 = read_log!(q!()); // Header
    let output_5 = read_log!(q!()); // No header

    mod bar {
        use super::*;

        pub fn baz() -> String {
            read_log!(q!())
        }
    }

    let output_6 = bar::baz(); // Header
    let output_7 = read_log!(q!()); // Header
    let output_8 = read_log!(q!()); // No header

    assert!(HEADER_RE.is_match(&output_1));
    assert!(!HEADER_RE.is_match(&output_2));

    assert!(HEADER_RE.is_match(&output_3));
    assert!(HEADER_RE.is_match(&output_4));
    assert!(!HEADER_RE.is_match(&output_5));

    assert!(HEADER_RE.is_match(&output_6));
    assert!(HEADER_RE.is_match(&output_7));
    assert!(!HEADER_RE.is_match(&output_8));
}
