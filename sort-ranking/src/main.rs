use crate::options::{parse_options, RankingType};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use structs::{RankingJson, RankingVideoData};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::sync::atomic::{AtomicU64, AtomicBool};
use std::sync::{atomic, Arc};
use std::time::Duration;

mod options;

fn main() {
    let options = parse_options();

    eprintln!("reading file...");
    let input_file = File::open(options.input_bin).unwrap();
    let input_file = BufReader::new(input_file);
    let mut ranking: RankingJson = bincode::deserialize_from(input_file).unwrap();

    eprintln!("generating ranking key...");
    let key_gen = key_generator_of(options.ranking_type);

    ranking.data.par_iter_mut()
        .for_each(|data| {
            data.ranking_counter = key_gen(data);
        })
        ;

    eprintln!("sorting...");
    ranking.data.par_sort_by_key(|data| { data.ranking_counter });

    eprintln!("writing...");
    let output_file = File::open(options.output_bin).unwrap();
    let output_file = BufWriter::new(output_file);
    bincode::serialize_into(output_file,&ranking).unwrap();
}

fn key_generator_of(for_type: RankingType) -> Box<dyn Fn(&RankingVideoData) -> u64 + Sync> {
    match for_type {
        RankingType::WatchSum => Box::new(|data| {
            data.length_seconds as u64 * data.view_counter as u64
        }),
        RankingType::WatchCnt => Box::new(|data| {
            data.view_counter as u64
        }),
        RankingType::WatchLng => Box::new(|data| {
            data.length_seconds as u64
        }),
    }
}
