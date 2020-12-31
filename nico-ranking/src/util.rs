use chrono::Duration;
use crossbeam::thread::Scope;
use std::path::PathBuf;
use std::env::current_exe;
use std::io::{BufReader, Read, BufRead};
use std::fs::metadata;

pub fn duration_to_string(dur: Duration) -> String {
    let std = dur.to_std().unwrap();
    let secs = std.as_secs();
    let nanos = std.subsec_nanos();
    format!("{} s {} ns", secs, nanos)
}

pub fn get_exec_path(name: &str) -> PathBuf {

    let path = if cfg!(not(windows)) {
        current_exe().unwrap().parent().unwrap().join(name)
    } else {
        current_exe().unwrap().parent().unwrap().join(format!("{}.exe", name))
    };

    metadata(&path).unwrap();
    path
}

pub fn pass_to_stream<'env, R: Read + Send + 'env>(
    s: &Scope<'env>,
    prefix: String,
    stream: R,
) {
    s.spawn(move |_| {
        for line in BufReader::new(stream).lines() {
            if let Ok(line) = line {
                println!("{}{}", prefix, line)
            }
        }
    });
}
