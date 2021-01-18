use chrono::{DateTime, Duration, FixedOffset, Utc, Local};
use reqwest::{StatusCode, Client};
use std::time::Instant;
use indicatif::{MultiProgress};
use crate::progress::ProgressStatus;
use crate::options::{Options};
use nico_snapshot_api::*;
use tokio::macros::support::Future;
use std::borrow::ToOwned;
use std::mem::swap;
use std::sync::mpsc::{Sender};
use crate::Packet;

const DATE_FORMAT: &str = "%Y/%m/%d";

pub(crate) struct Context<'a> {
    pub(crate) client: &'a reqwest::Client,
    pub(crate) last_req_time: Duration,
    pub(crate) progress: &'a MultiProgress,
    pub(crate) sender: Sender<Packet>,
}

impl<'a> Context<'a> {
    pub(crate) fn new(
        client: &'a reqwest::Client,
        progress: &'a MultiProgress,
        sender: Sender<Packet>,
    ) -> Context<'a> {
        Context {
            client,
            last_req_time: Duration::zero(),
            progress,
            sender
        }
    }

    pub(crate) fn get_wait_until(&self, _request_start: Instant) -> Instant {
        let last_req_time_since_now = Instant::now() + self.last_req_time.to_std().unwrap();

        return last_req_time_since_now
    }
}

pub(crate) async fn get_data(ctx: &mut Context<'_>, options: &Options) {
    let since = options.since;
    let until = options.until;
    let per = options.duration;

    let count = ((compute_until(until, &since.timezone()) - since).num_seconds() / per.num_seconds()) as u64;

    let mut progress = ProgressStatus::new(&ctx.progress);
    progress.set_count(0, count);

    let mut since_n = since;
    let mut until_n = std::cmp::min(compute_until(until, &since.timezone()), since_n + per);
    loop {
        progress.inc();
        progress.set_message(&format!("getting data since {} until {}",
                                      since_n.format(DATE_FORMAT),
                                      until_n.format(DATE_FORMAT),
        ));

        do_get_for_one_period(
            ctx,
            since_n,
            until_n,
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

async fn do_get_for_one_period(
    ctx: &mut Context<'_>,
    since: DateTime<FixedOffset>,
    until: DateTime<FixedOffset>,
) {
    let mut progress = ProgressStatus::new(&ctx.progress);

    let filter = FilterJson::Range(
        RangeFilter::start_time(since, until)
            .include_lower()
            .to_owned());


    let mut params = QueryParams::new("", RankingSorting::StartTime.increasing());
    params.with_fields(FieldName::all_values());
    params.set_filter(filter);
    params.set_limit(100);

    'outer: loop {
        progress.set_message(&format!("getting version before get..."));
        let pre_version = get_snapshot_version(ctx).await;

        let mut vec = Vec::<VideoInfo>::new();
        let mut loop_counter: u32 = 0;
        let mut got: u32 = 0;
        let mut full_count = 1;
        while got < full_count {
            if loop_counter % 100 == 100-1 {
                progress.set_msg_keeping_prefix(format!("getting version after 100 loop..."));
                let post_version = get_snapshot_version(ctx).await;
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
                1,
                || { format!("{}..{}#{}", since.format(DATE_FORMAT), until.format(DATE_FORMAT), got) },
                move |cli| { async move { params.get(&cli).await } }
            ).await;

            if vec.capacity() > json.meta.total_count {
                vec.reserve(json.meta.total_count - vec.len())
            }


            ctx.last_req_time = Duration::from_std(duration).unwrap();

            // set variables
            let len = json.data.len() as u32;
            full_count = json.meta.total_count as u32;
            got += len;
            for x in json.data {
                vec.push(x)
            }

            let until = ctx.get_wait_until(request_start);
            progress.set_msg_keeping_prefix(format!("waiting for server load reduction since {} until {}",
                                                    Local::now(),
                                                    Local::now() + Duration::from_std(until - Instant::now()).unwrap()));
            tokio::time::delay_until(ctx.get_wait_until(request_start).into())
                .await;
            let finished = len == 0;
            if finished {
                break;
            }
        }

        progress.set_message(&format!("getting version after get..."));
        let post_version = get_snapshot_version(ctx).await;
        if pre_version != post_version {
            progress.add_info(&format!("version was changed at the end: {}", since));
            continue
        } else {
            ctx.sender.send(Packet {
                last_modified: pre_version.last_modified,
                videos: vec
            }).unwrap();
            return
        }
    }
}

async fn get_snapshot_version(ctx: &mut Context<'_>) -> SnapshotVersion
{
    //*
    let mut progress = ProgressStatus::new(&ctx.progress);
    progress.set_prefix("snapshot_version");

    let (version, _) = http_request(
        ctx,
        &mut progress,
        1,
        1,
        || { format!("snapshot version") },
        move |cli| { async move { snapshot_version(&cli).await } },
    ).await;

    return version;
}

async fn http_request<'a, Fut: Future<Output = reqwest::Result<R>> + 'a, R>(
    ctx: &Context<'a>,
    progress: &mut ProgressStatus,
    minutes_for_wait_5xx: i64,
    minutes_for_wait_unknown: i64,
    get_name: impl Fn() -> String,
    get: impl Fn(&'a Client) -> Fut,
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
                            let err = format!("known 5xx status so wait for {} minutes: {}", minutes_for_wait_5xx, code);
                            progress.add_err(&format!("{}: {}", get_name(), err));
                            progress.set_msg_keeping_prefix(err);
                            tokio::time::delay_for(Duration::minutes(minutes_for_wait_5xx).to_std().unwrap()).await;
                            continue
                        }
                        _ => {
                            progress.add_err(&format!("unknown response: {}", code));

                            progress.set_msg_keeping_prefix(format!("unknown status so wait for {} minutes: {}", minutes_for_wait_unknown, code));
                            tokio::time::delay_for(Duration::minutes(minutes_for_wait_unknown).to_std().unwrap()).await;
                        }
                    }
                } else {
                    progress.add_err(&format!("unknown error: {}", err));
                }
            }
        }
    }
}
