use crate::options::{parse_options, RankingType};
use std::fs::{File, metadata};
use std::io::{BufReader, BufWriter, Read, Write};
use structs::{NewVideoInfo};
use structs::bincode_impl::{read_file, write_file};
use rayon::prelude::*;
use bincode::{ErrorKind, Deserializer, DefaultOptions};
use std::borrow::Borrow;
use std::time::Instant;

mod options;

fn main() {
    let options = parse_options();

    let start = Instant::now();
    eprintln!("reading file...");
    let input_file = File::open(&options.input_bin).unwrap();
    let size = metadata(&options.input_bin).unwrap().len();
    let mut input_file = BufReader::new(input_file);
    let mut videos: Vec<_> = read_file(&mut input_file, size as usize).collect();
    let key_gen = key_generator_of(options.ranking_type);
    eprintln!("reading file took {}s", (Instant::now() - start).as_secs_f64());

    let start = Instant::now();
    eprintln!("sorting...");
    videos.par_sort_by_key(|data| { key_gen(data) });
    eprintln!("sorting took {}s", (Instant::now() - start).as_secs_f64());

    let start = Instant::now();
    eprintln!("writing...");
    let output_file = File::create(options.output_bin).unwrap();
    let mut output_file = BufWriter::new(output_file);
    write_file(videos.iter(), &mut output_file);
    eprintln!("writing {}s", (Instant::now() - start).as_secs_f64());
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
