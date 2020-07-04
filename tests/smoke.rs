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
    pub static ref QFILE: Mutex<fs::File> = {
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

// TODO: Test cases for:
// - `q!(some_var)`
// - `q!(expr)`
// - Header line
//   - Changing file/function

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

#[test]
fn test_empty_call() {
    let line_before_call = line!();
    let output = read_log!(q!());
    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2);

    let header = lines[0];
    let log_line = lines[1];

    // TODO: Use capture groups to assert on captured values
    let header_re =
        Regex::new(r"^\[\d{2}:\d{2}:\d{2} tests/smoke\.rs smoke::test_empty_call:(\d+)\]$")
            .unwrap();
    let header_captures = header_re.captures(&header.trim()).unwrap();

    assert_eq!(
        header_captures.get(1).unwrap().as_str(),
        format!("{}", line_before_call + 1)
    );

    // TODO: Prefix log line with time elapsed since last header
    let log_line_re = Regex::new(r"^>$").unwrap();

    assert!(
        log_line_re.is_match(&log_line.trim()),
        format!("Log line mismatch: {:?}", log_line)
    );
}

// FIXME: This test changes the header interval for the *global* logger and
// could therefore affect other tests. Is there another way to reduce the
// interval just for this one test?
#[test]
fn test_header_interval() {
    let with_header_re =
        Regex::new(r"^\[\d{2}:\d{2}:\d{2} tests/smoke\.rs smoke::test_header_interval:\d+\]\n>$")
            .unwrap();
    let without_header_re = Regex::new(r"^>$").unwrap();

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

    assert!(with_header_re.is_match(&output_1));
    assert!(without_header_re.is_match(&output_2));
    assert!(with_header_re.is_match(&output_3));
    assert!(without_header_re.is_match(&output_4));
}

#[test]
fn test_header_on_function_or_module_change() {
    let with_header_re =
        Regex::new(r"^\[\d{2}:\d{2}:\d{2} tests/smoke\.rs smoke::test_header_on_function_or_module_change(.*)?:\d+\]\n>$")
            .unwrap();
    let without_header_re = Regex::new(r"^>$").unwrap();

    // Set long interval to ensure header is only triggered by function/module change
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

    assert!(with_header_re.is_match(&output_1));
    assert!(without_header_re.is_match(&output_2));

    assert!(with_header_re.is_match(&output_3));
    assert!(with_header_re.is_match(&output_4));
    assert!(without_header_re.is_match(&output_5));

    assert!(with_header_re.is_match(&output_6));
    assert!(with_header_re.is_match(&output_7));
    assert!(without_header_re.is_match(&output_8));
}
