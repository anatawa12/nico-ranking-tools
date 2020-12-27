use crate::options::parse_options;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use structs::{RankingBin, RankingVideoDataBin};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

mod options;

fn main() {
    let options = parse_options();
    let progress = ProgressBar::new(options.ranking_bins.len() as u64);
    set_style(&progress);

    //*
    let mut rankings: Vec<RankingBinCtx> = options.ranking_bins.par_iter()
        .map(|bin| {
            progress.set_message(&format!("reading {}", bin));
            progress.tick();
            let bin = File::open(bin).unwrap();
            let bin = BufReader::new(bin);
            let ranking: RankingBin = bincode::deserialize_from(bin).unwrap();
            progress.inc(1);
            RankingBinCtx{
                ranking,
                idx: 0,
            }
        })
        .collect();
    progress.finish();

    let progress = ProgressBar::new(rankings.iter().map(|x| x.ranking.data.len()).sum::<usize>() as u64);
    set_style(&progress);

    let mut out_csv = BufWriter::new(File::create(options.out_csv).unwrap());
    let mut rank_index = 1;

    writeln!(out_csv, "rank,ranking key,video id,get at,posted at,view count,video length").unwrap();

    loop {
        let ctx = match rankings.iter_mut()
            .filter(|ctx| !ctx.is_end())
            .max_by_key(|ctx| ctx.get_cur().map(|bin| bin.ranking_counter).unwrap()) {
            None => break,
            Some(a) => a
        };
        ctx.move_next();
        let cur = ctx.get_cur().unwrap();
        progress.set_message(&cur.content_id);

        writeln!(out_csv, "{},{},{},{},{},{},{}",
                 rank_index,
                 cur.ranking_counter,
                 cur.content_id,
                 ctx.ranking.meta.last_modified.unwrap(),
                 cur.start_time,
                 cur.view_counter,
                 cur.length_seconds,
        ).unwrap();

        rank_index += 1;
        progress.inc(1);
    }
}

struct RankingBinCtx {
    ranking: RankingBin,
    idx: usize,
}

impl RankingBinCtx {
    fn get_cur(&self) -> Option<&RankingVideoDataBin> {
        self.ranking.data.get(self.idx)
    }

    fn move_next(&mut self) {
        self.idx = self.idx + 1;
    }

    fn is_end(&self) -> bool {
        self.ranking.data.len() <= self.idx
    }
}

fn set_style(progress: &ProgressBar) {
    progress.set_style(ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-"));
}
