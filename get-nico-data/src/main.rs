mod progress;
mod options;

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
use nico_snapshot_api::{FilterJson, EqualFilter, RangeFilter, QueryParams, SortingWithOrder, RankingSorting, ResponseJson, snapshot_version, SnapshotVersion};
use bytes::Bytes;
use tokio::macros::support::Future;
use std::fs::File;
use std::mem::swap;

const DEFAULT_USER_AGENT: &str = concat!("view-counter-times-video-length-ranking-getting-daemon/", env!("CARGO_PKG_VERSION"));
const DATE_FORMAT: &str = "%Y/%m/%d";

fn main() {
    let options = parse_options();

    let client = reqwest::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build().unwrap();

    let progress = MultiProgress::new();
    let mut ctx = Context::new(&client, &progress);

    crossbeam::thread::scope(|s| {
        s.spawn(|s| {
            tokio::runtime::Builder::new()
                .threaded_scheduler()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    do_main(&mut ctx, options).await;
                    println!("finished main thread");
                })
        });
        s.spawn(|s| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            progress.join().unwrap();
            println!("finished!");
        });
    }).unwrap()
}

async fn do_main(ctx: &mut Context<'_>, options: Options) {
    let since = options.since;
    let until = options.until;
    let per = options.duration;
    let filter = options.filter;

    let base_dir = std::env::current_dir().unwrap().join("out");

    let count = ((compute_until(until, &since.timezone()) - since).num_seconds() / per.num_seconds()) as u64;

    let mut progress = ProgressStatus::new(&ctx.progress);
    progress.set_count(0, count);

    let mut since_n = since;
    let mut until_n = std::cmp::min(compute_until(until, &since.timezone()), since_n + per);
    loop {
        let dir = base_dir
            .join(since_n.format("%Y-%m-%d").to_string());

        std::fs::create_dir_all(&dir).unwrap();

        progress.inc();
        progress.set_message(&format!("getting data since {} until {}",
                                      since_n.format(DATE_FORMAT),
                                      until_n.format(DATE_FORMAT),
        ));

        do_get_for_one_period(
            ctx,
            since_n,
            until_n,
            dir.clone(),
            filter.clone(),
        ).await;
        swap(&mut until_n, &mut since_n);
        until_n = std::cmp::min(compute_until(until, &since.timezone()), since_n + per);
        if until_n - since_n < Duration::minutes(1) {
            break
        }
    }
}

fn compute_until<Tz: chrono::TimeZone>(until: Option<DateTime<Tz>>, tz: &Tz) -> DateTime<Tz> {
    match until {
        None => Utc::now().with_timezone(tz),
        Some(until) => until,
    }
}

struct Context<'a> {
    client: &'a reqwest::Client,
    max_req_time: Duration,
    last_req_time: Duration,
    progress: &'a MultiProgress,
}


impl<'a> Context<'a> {
    pub(crate) fn new(client: &'a reqwest::Client, progress: &'a MultiProgress) -> Context<'a> {
        Context {
            client,
            max_req_time: Duration::zero(),
            last_req_time: Duration::zero(),
            progress,
        }
    }

    pub(crate) fn get_wait_until(&self, request_start: Instant) -> Instant {
        let one_sec_since_req_start = request_start + Duration::seconds(1).to_std().unwrap();
        let last_req_time_since_now = Instant::now() + self.last_req_time.to_std().unwrap();

        return std::cmp::max(one_sec_since_req_start, last_req_time_since_now)
    }

    pub(crate) fn set_request_time(&mut self, dur: Duration) {
        if self.max_req_time < dur {
            self.max_req_time = dur;
        }
        self.last_req_time = dur;
    }
}

async fn do_get_for_one_period(
    ctx: &mut Context<'_>,
    since: DateTime<FixedOffset>,
    until: DateTime<FixedOffset>,
    dir: std::path::PathBuf,
    filter: Option<FilterJson>,
)
{
    let mut progress = ProgressStatus::new(&ctx.progress);

    let filter = {
        let range = FilterJson::Range(RangeFilter::start_time(since, until));
        if let Some(filter) = filter {
            FilterJson::And(vec![
                filter,
                FilterJson::Range(RangeFilter::start_time(since, until)),
            ])
        } else {
            range
        }
    };

    let mut params = QueryParams::new("", RankingSorting::StartTime.increasing());
    params.with_fields(&["contentId", "lengthSeconds", "startTime", "viewCounter"]);
    params.set_filter(filter);
    params.set_limit(100);

    'outer: loop {
        progress.set_message(&format!("getting version before get..."));
        let pre_version = get_snapshot_version(ctx, &dir).await;

        let mut loop_counter: u32 = 0;
        let mut got: u32 = 0;
        let mut full_count = 1;
        while got < full_count {
            if loop_counter % 100 == 100-1 {
                progress.set_msg_keeping_prefix(format!("getting version after 100 loop..."));
                let post_version = get_snapshot_version(ctx, &dir).await;
                if pre_version != post_version {
                    progress.add_info(&format!("version was changed when #{}: {}", got, since));
                    continue'outer
                }
            }
            loop_counter += 1;

            progress.set_count(got as u64, full_count as u64);
            progress.set_msg_keeping_prefix(format!("waiting response..."));

            let request_start = Instant::now();

            params.set_offset(got);

            let params = &params;
            let (json, duration) = http_request(
                ctx,
                &mut progress,
                5,
                5,
                move |cli| { async move { params.get(&cli).await } }
            ).await;

            ctx.set_request_time(Duration::from_std(duration).unwrap());

            progress.set_msg_keeping_prefix(format!("writing to file..."));

            // write to file
            let path = dir.join(format!("ranking_{:06}.json", got));
            match File::create(&path)
                .map(|file| BufWriter::new(file))
                .map(|writer| serde_json::to_writer(writer, &json)) {
                Ok(_) => {}
                Err(err) => progress.add_err(&format!("cannot write file: {}: {}", path.display(), err)),
            }

            // set variables
            let data = &json.data;
            full_count = json.meta.total_count as u32;
            got += data.len() as u32;
            let finished = data.len() == 0;

            progress.set_msg_keeping_prefix(format!("waiting for server load reduction..."));
            tokio::time::delay_until(ctx.get_wait_until(request_start).into())
                .await;
            if finished {
                break;
            }
        }

        progress.set_message(&format!("getting version after get..."));
        let post_version = get_snapshot_version(ctx, &dir).await;
        if pre_version != post_version {
            progress.add_info(&format!("version was changed at the end: {}", since));
            continue
        } else {
            break
        }
    }
}

async fn get_snapshot_version(ctx: &mut Context<'_>, dir: &std::path::PathBuf) -> SnapshotVersion
{
    //*
    let mut progress = ProgressStatus::new(&ctx.progress);
    let (version, _) = http_request(
        ctx,
        &mut progress,
        1,
        1,
        move |cli| { async move { snapshot_version(&cli).await } },
    ).await;

    progress.set_prefix("snapshot_version");
    progress.set_msg_keeping_prefix(format!("writing to file..."));
    // write to file
    let path = dir.join("version.json");

    match File::create(&path)
        .map(|file| BufWriter::new(file))
        .map(|writer| serde_json::to_writer(writer, &version)) {
        Ok(_) => {}
        Err(err) => progress.add_err(&format!("cannot write file: {}: {}", path.display(), err)),
    }

    return version;
}

async fn http_request<'a, Fut: Future<Output = reqwest::Result<R>> + 'a, R>(
    ctx: &Context<'a>,
    progress: &mut ProgressStatus,
    minutes_for_wait_5xx: i64,
    minutes_for_wait_unknown: i64,
    get: impl Fn(&'a Client) -> Fut
) -> (R, tokio::time::Duration) {
    loop {
        progress.set_msg_keeping_prefix(&format!("waiting response..."));
        let request_start = Instant::now();
        match get(ctx.client).await {
            Ok(value) => {
                let request_end = Instant::now();
                return (value, request_end - request_start);
            }
            Err(err) => {
                if let Some(code) = err.status() {
                    match code {
                        | StatusCode::INTERNAL_SERVER_ERROR
                        | StatusCode::NOT_IMPLEMENTED
                        | StatusCode::BAD_GATEWAY
                        | StatusCode::SERVICE_UNAVAILABLE
                        | StatusCode::GATEWAY_TIMEOUT
                        | StatusCode::HTTP_VERSION_NOT_SUPPORTED
                        | StatusCode::VARIANT_ALSO_NEGOTIATES
                        | StatusCode::INSUFFICIENT_STORAGE
                        | StatusCode::LOOP_DETECTED
                        | StatusCode::NOT_EXTENDED
                        | StatusCode::NETWORK_AUTHENTICATION_REQUIRED
                        => {
                            progress.set_msg_keeping_prefix(format!("known 5xx status so wait for {} minutes", minutes_for_wait_5xx));
                            tokio::time::delay_for(Duration::minutes(minutes_for_wait_5xx).to_std().unwrap()).await;
                            continue
                        }
                        _ => {
                            progress.add_err(&format!("unknown response: {}", code));

                            progress.set_msg_keeping_prefix(format!("unknown status so wait for {} minutes", minutes_for_wait_unknown));
                            tokio::time::delay_for(Duration::minutes(minutes_for_wait_unknown).to_std().unwrap()).await;
                        }
                    }
                } else {
                    Err(err).unwrap()
                }
            }
        }
    }
}
