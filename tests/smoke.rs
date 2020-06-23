#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate q;

use regex::Regex;
use std::env;
use std::fs;
use std::io::Read;
use std::sync::Mutex;

lazy_static! {
    pub static ref QFILE: Mutex<fs::File> = {
        Mutex::new(
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .append(true)
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
//   - Regular interval

#[test]
fn test_empty_call() {
    // TODO: Pull out setup/log line checking into a macro for other tests
    let mut file = QFILE.lock().unwrap();
    let mut output = String::new();

    file.read_to_string(&mut output).unwrap();
    output.clear();

    q!();

    file.read_to_string(&mut output).unwrap();

    let lines: Vec<&str> = output.lines().collect();

    assert_eq!(lines.len(), 2);

    let header = lines[0];
    let log_line = lines[1];

    // TODO: Use capture groups to assert only on captured values
    let header_re =
        Regex::new(r"^\[\d{2}:\d{2}:\d{2} tests/smoke\.rs smoke::test_empty_call:35\]$").unwrap();
    let log_line_re = Regex::new(r"^\d+\.\d+s >$").unwrap();

    assert!(
        header_re.is_match(&header.trim()),
        format!("Header mismatch: {:?}", header)
    );
    assert!(
        log_line_re.is_match(&log_line.trim()),
        format!("Log line mismatch: {:?}", log_line)
    );
}
