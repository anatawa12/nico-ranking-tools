mod progress;
mod options;
mod get_data_from_server;
mod output;

use chrono::{DateTime, FixedOffset, TimeZone};
use indicatif::{MultiProgress};
use crate::options::{parse_options};
use nico_snapshot_api::VideoInfo;
use std::sync::mpsc;
use crate::get_data_from_server::{get_data, Context};

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
                    get_data(&mut ctx, &options).await;
                    ctx.sender.send(Packet{ last_modified: FixedOffset::east(0).timestamp(0, 0), videos: Vec::new() }).unwrap();
                    eprintln!("finished main thread");
                })
        });
        s.spawn(|_| {
            output::run(receiver, &options);
        });
        s.spawn(|_| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            progress.join().unwrap();
            eprintln!("finished!");
        });
    }).unwrap()
}
