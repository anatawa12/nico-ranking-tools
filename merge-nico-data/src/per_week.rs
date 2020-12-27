use std::path::PathBuf;
use crate::options::Options;
use crate::progress::Progress;
use structs::{RankingVideoData, VersionJson};
use crate::common::{process_a_week, remove_a_week, output_to_bincode};
use std::fs::File;

pub(crate) fn process_per_weeks(
    weeks: &Vec<PathBuf>,
    options: &Options,
    progress: Progress,
) {
    progress.set_length(weeks.len() as u64);
    for week_dir in weeks.iter() {
        progress.inc(1);
        progress.set_message(&format!("processing {}", week_dir.display()));

        let version_json: VersionJson = serde_json::from_reader(
            File::open(week_dir.join("version.json")).unwrap()).unwrap();

        let mut data: Vec<RankingVideoData> = Vec::new();

        if !process_a_week(week_dir, &mut data, progress.create_child(1)) {
            progress.println(format!("no much for {}", week_dir.display()));
            continue
        }

        let merged = week_dir.join("merged.bin");
        progress.set_message(&format!("writing {}", merged.display()));
        output_to_bincode(merged, data, version_json.last_modified);

        if options.remove_old {
            remove_a_week(week_dir, progress.create_child(1));
        }
    }
}
