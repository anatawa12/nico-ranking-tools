mod progress;
mod options;
mod get_data_from_server;
mod output;

use chrono::{DateTime, FixedOffset, TimeZone};
use std::io::{stdout};
use indicatif::{MultiProgress};
use crate::options::{parse_options};
use nico_snapshot_api::VideoInfo;
use std::sync::mpsc;
use crate::get_data_from_server::{get_data, Context};
use std::path::Path;
use std::fs::{create_dir_all, File};

const DEFAULT_USER_AGENT: &str = concat!("view-counter-times-video-length-ranking-getting-daemon/", env!("CARGO_PKG_VERSION"));

struct Packet {
    last_modified: DateTime<FixedOffset>,
    videos: Vec<VideoInfo>,
}

fn main() {
    let options = parse_options();

    let client = reqwest::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build().unwrap();

    let progress = MultiProgress::new();
    let out_file = options.out.clone();

    crossbeam::thread::scope(|s| {
        let (sender, receiver) = mpsc::channel::<Packet>();
        s.spawn(|_| {
            tokio::runtime::Builder::new()
                .threaded_scheduler()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let mut ctx = Context::new(&client, &progress, sender);
                    get_data(&mut ctx, options).await;
                    ctx.sender.send(Packet{ last_modified: FixedOffset::east(0).timestamp(0, 0), videos: Vec::new() }).unwrap();
                    eprintln!("finished main thread");
                })
        });
        s.spawn(|_| {
            match &out_file {
                None => {
                    output::run(receiver, stdout())
                }
                Some(name) => {
                    create_dir_all(Path::new(&name).parent().unwrap()).unwrap();
                    output::run(receiver, File::create(name).unwrap())
                }
            }
        });
        s.spawn(|_| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            progress.join().unwrap();
            eprintln!("finished!");
        });
    }).unwrap()
}
