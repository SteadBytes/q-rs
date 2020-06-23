#[macro_use]
mod macros;

use chrono::prelude::*;
use std::env;
use std::fs;
use std::io::prelude::*;

pub fn q_log(file_path: &str, func_path: &str, lineno: u32) {
    // FIXME: Actually set elapsed time
    let log_line = format!(
        "[{} {} {}:{}]\n0.000s >",
        Utc::now().time().format("%H:%M:%S").to_string(),
        file_path,
        func_path,
        lineno
    );
    write_to_log(&log_line);
}

pub fn write_to_log(s: &str) {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(env::temp_dir().join("q"))
        .expect("Unable to open qpath");

    writeln!(file, "{}", s).expect("Unable to write to qpath");
}
