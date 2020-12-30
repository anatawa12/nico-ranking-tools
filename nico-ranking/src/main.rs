use options::{parse_options, Options, Phase};
use std::path::{Path};
use std::process::{Command, Stdio};
use util::*;
use tempfile::{NamedTempFile};
use std::io::{Write, BufWriter};
use std::fs::{read_dir, create_dir_all};
use crossbeam::thread;
use std::env::current_dir;
use std::ffi::OsStr;
use ansi_term::Color;

mod options;
mod util;

fn main() {
    #[cfg(windows)]
    let _ = ansi_term::enable_ansi_support();
    let options = parse_options();

    let work_dir = current_dir().unwrap();

    if options.phase_since <= Phase::GetNicoData {
        run_get_nico_data(&options, &work_dir);
    }

    let merged = work_dir.join("merged");
    create_dir_all(&merged).unwrap();

    if options.phase_since <= Phase::MergeNicoData {
        run_merge_nico_data(work_dir.join("out"), &merged);
    }

    let sorted = work_dir.join("sorted");
    create_dir_all(&sorted).unwrap();

    if options.phase_since <= Phase::SortRanking {
        let colors = vec![
            Color::Red,
            Color::Green,
            //Color::Yellow,  not good when background is white
            //Color::Blue,    not good when background is black
            Color::Purple,
            Color::Cyan,
        ];
        thread::scope(|s| {
            let mut color_index: usize = 0;
            for entry in read_dir(&merged).unwrap() {
                if let Ok(entry) = entry {
                    if entry.path().extension() == Some(OsStr::new("bin")) {
                        let options = &options;
                        let merged_bin = entry.path();
                        let sorted_bin = sorted.join(entry.file_name());
                        let color = colors[color_index % colors.len()];
                        s.spawn(move |_| {
                            run_sort_ranking(options, color, &merged_bin, &sorted_bin);
                        });
                        color_index += 1;
                    }
                }
            }
        }).unwrap();
    }

    let sorted_bins: Vec<_> = read_dir(sorted).unwrap()
        .filter_map(|it| it.ok())
        .filter(|entry| entry.path().extension() == Some(OsStr::new("bin")))
        .map(|entry| entry.path())
        .collect();

    let ranking_csv = work_dir.join("ranking.csv");

    if options.phase_since <= Phase::MergeRankings {
        run_merge_rankings(&ranking_csv, &sorted_bins);
    }

    let html_dir = work_dir.join("html");

    if options.phase_since <= Phase::HtmlGen {
        run_html_gen(&ranking_csv, &html_dir);
    }
}

fn run_get_nico_data(options: &Options, work_dir: &Path) {
    const DATE_FORMAT: &str = "%Y/%m/%d";

    println!("running get-nico-data...");

    let mut cmd = Command::new(get_exec_path("get-nico-data"));

    cmd.current_dir(work_dir);

    if let Some(since) = options.since {
        cmd.args(&["--since", &since.format(DATE_FORMAT).to_string()]);
    }
    if let Some(since) = options.since {
        cmd.args(&["--since", &since.format(DATE_FORMAT).to_string()]);
    }
    if let Some(duration) = options.duration {
        cmd.args(&["--since", &duration_to_string(duration)]);
    }
    if let Some(json) = &options.filter {
        let mut named = NamedTempFile::new().unwrap();
        serde_json::to_writer(BufWriter::new(&mut named), json).unwrap();
        named.flush().unwrap();
        cmd.arg("--filter");
        cmd.arg(named.path());
    }

    let mut cmd = cmd.spawn().unwrap();
    let exit = cmd.wait().unwrap();
    assert!(exit.success(), "get-nico-data returns non zero value");
}

fn run_merge_nico_data<P1: AsRef<Path>, P2: AsRef<Path>>(out_dir: P1, merged_dir: P2) {
    println!("running merge-nico-data...");

    let mut cmd = Command::new(get_exec_path("merge-nico-data"));

    cmd.current_dir(merged_dir);

    // merge as possible as
    cmd.arg("-a");
    cmd.arg(out_dir.as_ref());

    let mut cmd = cmd.spawn().unwrap();
    let exit = cmd.wait().unwrap();
    assert!(exit.success(), "merge-nico-data returns non zero value");
}

fn run_sort_ranking<P1: AsRef<Path>, P2: AsRef<Path>>(
    options: &Options,
    color: Color,
    merged_bin: P1,
    sorted_bin: P2,
) {
    println!("running sort-ranking...");
    let mut cmd = Command::new(get_exec_path("sort-ranking"));

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd.arg(merged_bin.as_ref());
    cmd.arg(sorted_bin.as_ref());
    cmd.arg(&options.ranking_type);

    let name = sorted_bin.as_ref().file_name().unwrap().to_str().unwrap().to_string();
    let prefix = color.paint(format!("{}: ", name)).to_string();

    let mut cmd = cmd.spawn().unwrap();
    let stdout = cmd.stdout.take().unwrap();
    let stderr = cmd.stderr.take().unwrap();

    thread::scope(|s| {
        pass_to_stream(s, prefix.clone(), stderr);
        pass_to_stream(s, prefix.clone(), stdout);
    }).unwrap();
    let exit = cmd.wait().unwrap();
    assert!(exit.success(), "sort-ranking returns non zero value");
    println!("{}finished!", prefix)
}

fn run_merge_rankings<P1: AsRef<Path>, P2: AsRef<Path>>(
    ranking_csv: P1,
    sorted_bins: &[P2],
) {
    println!("running merge-rankings...");
    let mut cmd = Command::new(get_exec_path("merge-rankings"));

    cmd.arg(ranking_csv.as_ref());
    for x in sorted_bins {
        cmd.arg(x.as_ref());
    }

    let mut cmd = cmd.spawn().unwrap();

    let exit = cmd.wait().unwrap();
    assert!(exit.success(), "merge-rankings returns non zero value");
}

fn run_html_gen<P1: AsRef<Path>, P2: AsRef<Path>>(
    ranking_csv: P1,
    html_dir: P2,
) {
    println!("running html-gen...");
    let mut cmd = Command::new(get_exec_path("html-gen"));

    cmd.arg(ranking_csv.as_ref());
    cmd.arg(html_dir.as_ref());

    let mut cmd = cmd.spawn().unwrap();

    let exit = cmd.wait().unwrap();
    assert!(exit.success(), "html-gen returns non zero value");
}
