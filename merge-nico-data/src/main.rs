mod options;
mod merging;
mod util;
mod common;
mod progress;
mod per_week;

use std::thread;
use regex::Regex;
use lazy_static::lazy_static;
use indicatif::{MultiProgress};
use crate::progress::Progress;
use crate::per_week::process_per_weeks;

lazy_static! {
    static ref WEEK_DIR_NAME_REGEX: Regex = Regex::new(r#"^\d{4}-\d{2}-\d{2}$"#).unwrap();
    static ref MULTI: MultiProgress = MultiProgress::new();
}

fn main() {
    let options = options::parse_options();

    let weeks = util::sorted_ls_matches_regex(&options.out_dir, &*WEEK_DIR_NAME_REGEX);
    let progress = Progress::new(&MULTI, 1);

    let _ = thread::spawn(move || {
        if options.merge_all {
            merging::process_merge_weeks(
                &weeks,
                &options,
                progress,
            );
        } else {
            process_per_weeks(
                &weeks,
                &options,
                progress,
            );
        }
    });

    MULTI.join_and_clear().unwrap();
}
