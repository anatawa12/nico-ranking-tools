use chrono::{DateTime, FixedOffset};
use std::fs::File;
use std::path::{PathBuf};
use crate::options::Options;
use crate::structs::{VersionJson, RankingVideoData};
use crate::common::{output_to_json, process_a_week, remove_a_week};
use crate::progress::Progress;

pub(crate) fn process_merge_weeks(
    weeks: &Vec<PathBuf>,
    options: &Options,
    mut progress: Progress,
) {
    let mut data: Vec<RankingVideoData> = Vec::new();
    let mut will_removes: Vec<PathBuf> = Vec::new();
    let mut last_modified: Option<DateTime<FixedOffset>> = None;
    let mut ranking_index = 0;

    progress.set_length(weeks.len() as u64);
    for week_dir in weeks.iter() {
        progress.set_message(&format!("processing {}", week_dir.display()));
        progress.inc(1);

        let version_json: VersionJson = serde_json::from_reader(
            File::open(week_dir.join("version.json")).unwrap()).unwrap();

        if changed_version(&version_json, &last_modified) {
            let merged_name = format!("merged_{}.json", ranking_index);
            output_to_json(merged_name, data, last_modified.unwrap());
            ranking_index += 1;
            data = Vec::new();
        }

        last_modified = Some(version_json.last_modified);

        if !process_a_week(week_dir, &mut data, progress.create_child(1)) {
            progress.println(format!("no much for {}", week_dir.display()));
            continue
        }
        will_removes.append(&mut vec![week_dir.clone()]);
    }

    if last_modified.is_some() {
        let merged_name = format!("merged_{}.json", ranking_index);
        output_to_json(merged_name, data, last_modified.unwrap());
    }

    progress.new_generation(weeks.len() as u64);

    if options.remove_old {
        for week_dir in will_removes.iter() {
            progress.inc(1);
            progress.set_message(&format!("removing {}", week_dir.display()));
            remove_a_week(week_dir, progress.create_child(1));
        }
    }
}

fn changed_version(version_json: &VersionJson, last_modified: &Option<DateTime<FixedOffset>>) -> bool {
    return last_modified != &None && last_modified.unwrap() != version_json.last_modified;
}
