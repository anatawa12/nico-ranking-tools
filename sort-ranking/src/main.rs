use crate::options::{parse_options, RankingType};
use std::fs::{File};
use std::io::{BufReader, BufWriter, Read};
use structs::{NewVideoInfo};
use rayon::prelude::*;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use crate::progress_reader::ProgressReader;
use std::cmp::Ordering;

mod options;
mod option_expr_parser;
mod progress_reader;

fn main() {
    let options = parse_options();

    let start = Instant::now();
    eprintln!("reading file...");
    let input_bin_size = std::fs::metadata(&options.input_bin).unwrap().len();
    let mut input_bin = File::open(&options.input_bin).unwrap();
    let mut videos: Vec<NewVideoInfo> = get_videos(&mut input_bin, input_bin_size);
    let key_gen = key_generator_of(options.ranking_type);
    eprintln!("reading file took {}s", (Instant::now() - start).as_secs_f64());

    if let Some(filter) = options.filter {
        let start = Instant::now();
        eprintln!("filtering...");

        videos.retain(filter);

        eprintln!("filtering took {}s", (Instant::now() - start).as_secs_f64());
    }

    let start = Instant::now();
    eprintln!("sorting...");
    videos.par_sort_by_key(|data| key_gen(data).reversing());
    eprintln!("sorting took {}s", (Instant::now() - start).as_secs_f64());

    let start = Instant::now();
    eprintln!("writing...");
    let output_file = File::create(options.output_bin).unwrap();
    let mut output_file = BufWriter::new(output_file);
    bincode::serialize_into(&mut output_file, &videos).unwrap();
    eprintln!("writing {}s", (Instant::now() - start).as_secs_f64());
}

fn get_videos<R: Read>(input_bin: R, input_bin_size: u64) -> Vec<NewVideoInfo> {
    let progress = ProgressBar::new(input_bin_size);
    progress.set_message("reading binary...");
    progress.enable_steady_tick(10);
    set_style(&progress);
    let mut input_bin = ProgressReader::new(&progress, input_bin);
    let mut input_bin = BufReader::new(input_bin);

    return bincode::deserialize_from(&mut input_bin).unwrap();
}

fn set_style(progress: &ProgressBar) {
    progress.set_style(ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40blue} {pos:>7}/{len:7} ({percent}%) {msg}")
        .progress_chars("##-"));
}

fn key_generator_of(for_type: RankingType) -> Box<dyn Fn(&NewVideoInfo) -> u64 + Sync> {
    match for_type {
        RankingType::WatchSum => Box::new(|data| {
            data.length_seconds.as_secs() * data.view_counter as u64
        }),
        RankingType::WatchCnt => Box::new(|data| {
            data.view_counter as u64
        }),
        RankingType::WatchLng => Box::new(|data| {
            data.length_seconds.as_secs()
        }),
    }
}

struct Reversing<T: Ord>(T);

impl <T : Ord> Eq for Reversing<T> {
}

impl <T : Ord> PartialEq<Self> for Reversing<T> {
    fn eq(&self, other: &Reversing<T>) -> bool {
        self.0 == other.0
    }
}

impl <T : Ord> Ord for Reversing<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl <T : Ord> PartialOrd<Self> for Reversing<T> {
    fn partial_cmp(&self, other: &Reversing<T>) -> Option<Ordering> {
        self.0.partial_cmp(&other.0).map(|x| x.reverse())
    }
}

trait ReversingOrd : Ord + Sized {
    fn reversing(self) -> Reversing<Self>;
}

impl <T: Ord> ReversingOrd for T {
    fn reversing(self) -> Reversing<Self> {
        Reversing(self)
    }
}
