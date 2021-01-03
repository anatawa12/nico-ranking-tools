mod progress;
mod options;
mod get_data_from_server;
mod output;

use chrono::{DateTime, Duration, FixedOffset, Utc, TimeZone};
use std::process::exit;
use url::{Url};
use reqwest::{StatusCode, IntoUrl, RequestBuilder, Client, Error};
use std::time::Instant;
use serde_json::{Value as JValue, Map};
use std::io::{stdout, Write, BufWriter};
use indicatif::{ProgressBar, MultiProgress};
use std::fmt::Display;
use crate::progress::ProgressStatus;
use crate::options::{parse_options, Options};
use nico_snapshot_api::{FilterJson, EqualFilter, RangeFilter, QueryParams, SortingWithOrder, RankingSorting, ResponseJson, snapshot_version, SnapshotVersion, FieldName, VideoInfo};
use bytes::Bytes;
use tokio::macros::support::Future;
use std::fs::File;
use std::mem::swap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use crate::get_data_from_server::{get_data, Context};

const DEFAULT_USER_AGENT: &str = concat!("view-counter-times-video-length-ranking-getting-daemon/", env!("CARGO_PKG_VERSION"));
const DATE_FORMAT: &str = "%Y/%m/%d";

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
        s.spawn(|s| {
            tokio::runtime::Builder::new()
                .threaded_scheduler()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let mut ctx = Context::new(&client, &progress, sender);
                    get_data(&mut ctx, options).await;
                    ctx.sender.clone();
                    eprintln!("finished main thread");
                })
        });
        s.spawn(|s| {
            output::run(receiver, stdout())
        });
        s.spawn(|s| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            progress.join().unwrap();
            eprintln!("finished!");
        });
    }).unwrap()
}
