use std::path::{Path, PathBuf};
use structs::{RankingVideoData, RankingMeta, RankingJson};
use chrono::{FixedOffset, DateTime};
use std::fs::File;
use lazy_static::lazy_static;
use crate::util::sorted_ls_matches_regex;
use regex::Regex;
use crate::progress::Progress;
use std::fs;
use std::io::{BufWriter, BufReader};

lazy_static! {
    pub(crate) static ref RANKING_JSON_NAME_REGEX: Regex = Regex::new(r#"^ranking_\d{6}\.json$"#).unwrap();
    pub(crate) static ref WEEK_DIR_NAME_REGEX: Regex = Regex::new(r#"^\d{4}-\d{2}-\d{2}$"#).unwrap();
}

pub(crate) fn output_to_json<P: AsRef<Path>>(
    merged_name: P,
    data: Vec<RankingVideoData>,
    last_modified: DateTime<FixedOffset>,
) {
    let merged_file = File::create(merged_name).unwrap();
    serde_json::to_writer(BufWriter::new(merged_file), &RankingJson {
        meta: RankingMeta {
            total_count: data.len() as u64,
            last_modified: Some(last_modified),
        },
        data
    }).unwrap();
}

pub(crate) fn process_a_week(
    week_dir: &PathBuf,
    data: &mut Vec<RankingVideoData>,
    progress: Progress,
) -> bool {
    let rankings: Vec<PathBuf> = sorted_ls_matches_regex(week_dir, &*RANKING_JSON_NAME_REGEX);

    progress.set_length(rankings.len() as u64);
    for ranking in rankings.iter() {
        progress.inc(1);
        progress.set_message(&format!("processing {}", ranking.display()));
        let mut ranking_data: RankingJson = serde_json::from_reader(BufReader::new(File::open(ranking).unwrap())).unwrap();
        if ceil_div(ranking_data.meta.total_count, 100) > rankings.len() as u64 {
            return false; // no much data
        }

        data.append(&mut ranking_data.data);
    }

    return true;
}

pub(crate) fn remove_a_week(
    week_dir: &PathBuf,
    progress: Progress,
) {
    let rankings: Vec<PathBuf> = sorted_ls_matches_regex(week_dir, &*RANKING_JSON_NAME_REGEX);
    progress.set_length(rankings.len() as u64 + 1);

    progress.set_message(&format!("removing {}", week_dir.join("version.json").display()));
    progress.inc(1);
    let _ = fs::remove_file(week_dir.join("version.json"));

    for ranking_json in rankings.iter() {
        progress.set_message(&format!("removing {}", ranking_json.display()));
        progress.inc(1);
        let _ = fs::remove_file(ranking_json);
    }
}

fn ceil_div<T>(p0: T, p1: T) -> T
    where T : std::ops::Add<Output = T>
    + std::ops::Sub<Output = T>
    + std::ops::Div<Output = T>
    + num::traits::One + Copy {
    return (p0 + p1 - T::one()) / p1;
}
