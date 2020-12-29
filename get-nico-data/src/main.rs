use chrono::{DateTime, Duration, FixedOffset, Utc, TimeZone};
use std::process::exit;
use url::{Url};
use reqwest::StatusCode;
use std::time::Instant;
use serde_json::{Value as JValue, Map};
use std::io::{stdout, Write};
use bytes::Bytes;

macro_rules! exiting_errf {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        exit(-1)
    })
}

const DEFAULT_USER_AGENT: &str = concat!("view-counter-times-video-length-ranking-getting-daemon/", env!("CARGO_PKG_VERSION"));
const SEARCH_SNAPSHOT_V2_ENDPOINT: &str = "https://api.search.nicovideo.jp/api/v2/snapshot/video/contents/search";
const SEARCH_SNAPSHOT_V2_VERSION: &str = "https://api.search.nicovideo.jp/api/v2/snapshot/version";
const DATE_FORMAT: &str = "%Y/%m/%d";
const DATE_FORMAT_WITH_TIME: &str = "%Y/%m/%d %H:%M:%S";
const ZERO_O_CLOCK_POSTFIX: &str = " 00:00:00";

#[tokio::main]
async fn main() {
    let jst_timezone = FixedOffset::east(9*3600);
    let args: Vec<String> = std::env::args().collect();
    let args = match args.len() - 1 {
        3 => {
            args
        },
        0 => vec![
            "dummy".to_string(),
            "2007/03/06".to_string(),
            Utc::now().with_timezone(&jst_timezone).format(DATE_FORMAT).to_string(),
            "1 weeks".to_string(),
        ],
        _ => {
            eprintln!("{} arguments are passed, expects 3 or zero.", args.len());
            eprintln!("{} <since> <until> <per>", args.get(1).map_or("get-nico-data", |str| str as &str));
            exit(-1);
        },
    };
    let since = jst_timezone.datetime_from_str(&format!("{}{}", args[1], ZERO_O_CLOCK_POSTFIX), DATE_FORMAT_WITH_TIME)
        .unwrap_or_else(|e| exiting_errf!("invalid #1: {}: {}", e, &args[1]));
    let until = if args[2] == "now" {
        None
    } else {
        Some(jst_timezone.datetime_from_str(&format!("{}{}", args[2], ZERO_O_CLOCK_POSTFIX), DATE_FORMAT_WITH_TIME)
            .unwrap_or_else(|e| exiting_errf!("invalid #2: {}: {}", e, &args[2])))
    };
    let per: Duration = Duration::from_std(parse_duration::parse(&args[3]).unwrap())
        .unwrap_or_else(|_| exiting_errf!("invalid #3: {}", &args[3]));

    let client = reqwest::Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build().unwrap();

    let mut ctx = Context {
        client: &client,
        max_req_time: Duration::zero(),
        last_req_time: Duration::zero(),
        infos: Vec::new(),
        statuses: Vec::new(),
    };

    let base_dir = std::env::current_dir().unwrap().join("out");

    let count = ((since - compute_until(until, &since.timezone())).num_seconds() / per.num_seconds()) as u32;

    let status_idx = ctx.add_status("");
    ctx.set_max(status_idx, count);

    let mut since_n = since;
    let mut until_n = std::cmp::min(compute_until(until, &since.timezone()), since_n + per);
    loop {
        let dir = base_dir
            .join(since_n.format("%Y-%m-%d").to_string());

        std::fs::create_dir_all(&dir).unwrap();

        ctx.set_msg(status_idx, &format!("getting data since {} until {} to {}",
                                         since_n.format(DATE_FORMAT),
                                         until_n.format(DATE_FORMAT),
                                         dir.display()));

        do_get_for_one_period(&mut ctx, since_n, until_n, dir.clone()).await;
        let until_n1 = until_n;
        until_n = std::cmp::min(compute_until(until, &since.timezone()), since_n + per);
        since_n = until_n1;
        if since_n >= until_n {
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
    infos: Vec<String>,
    statuses: Vec<Status>,
}


impl<'a> Context<'a> {
    pub(crate) fn add_err(&mut self, p0: &str) {
        self.infos.insert(self.infos.len(), format!("err: {}", p0));
        self.redraw();
    }
    pub(crate) fn add_info(&mut self, p0: &str) {
        self.infos.insert(self.infos.len(), format!("inf: {}", p0));
        self.redraw();
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

    pub(crate) fn add_status(&mut self, msg: &str) -> usize
    {
        let idx = self.statuses.len();
        self.statuses.insert(idx, Status { message: msg.to_string(), max: 1, cur: 0 });
        self.redraw();
        idx
    }

    pub(crate) fn set_msg(&mut self, idx: usize, msg: &str)
    {
        self.statuses[idx].message = msg.to_string();
        self.redraw();
    }

    pub(crate) fn set_max(&mut self, idx: usize, max: u32)
    {
        self.statuses[idx].max = max;
        self.redraw();
    }

    pub(crate) fn set_cur(&mut self, idx: usize, cur: u32)
    {
        self.statuses[idx].cur = cur;
        self.redraw();
    }

    pub(crate) fn pop_status(&mut self, idx: usize)
    {
        if self.statuses.len() != idx + 1 {
            panic!("invalid status: popping #{}", idx);
        }
        self.statuses.remove(idx);
    }

    pub(crate) fn redraw(&self) {
        let _ = crossterm::execute!(
            stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
        );
        for x in &self.infos {
            println!("{}", x)
        }
        println!("====");
        for x in &self.statuses {
            if x.max == u32::max_value() {
                println!("{}", &x.message);
            } else {
                let percentage = x.cur as f64 * 100.0 / x.max as f64;
                println!("{:120} {:-3.3}%", &x.message, percentage);
            }
        }
    }
}

struct Status {
    message: String,
    max: u32,
    cur: u32,
}

async fn do_get_for_one_period<Tz : chrono::TimeZone>(
    ctx: &mut Context<'_>,
    since: DateTime<Tz>,
    until: DateTime<Tz>,
    dir: std::path::PathBuf,
)
    where Tz::Offset: std::fmt::Display,
{
    let counting_status_idx = ctx.add_status("");
    let whats_now_status_idx = ctx.add_status("preprocessing");
    ctx.set_max(whats_now_status_idx, u32::max_value());

    let endpoint = Url::parse_with_params(
        SEARCH_SNAPSHOT_V2_ENDPOINT,
        &[
            ("q", ""),
            ("_sort", "+startTime"),
            ("fields", "contentId,lengthSeconds,startTime,viewCounter"),
            ("filters[startTime][gte]", &since.to_rfc3339()),
            ("filters[startTime][lt]", &until.to_rfc3339()),
            ("_limit", "100"),
        ],
    ).unwrap().to_string();

    'outer: loop {
        ctx.set_msg(whats_now_status_idx, &format!("getting version before get..."));
        let pre_version = get_snapshot_version(ctx, &dir).await;

        let mut loop_counter: u32 = 0;
        let mut got = 0;
        let mut full_count = 1;
        while got < full_count {
            if loop_counter % 100 == 100-1 {
                ctx.set_msg(whats_now_status_idx, &format!("getting version after 100 loop..."));
                let post_version = get_snapshot_version(ctx, &dir).await;
                if *pre_version != *post_version {
                    ctx.add_info(&format!("version was changed when #{}: {}", got, since));
                    continue'outer
                }
            }
            loop_counter += 1;

            ctx.set_msg(counting_status_idx, &format!("getting# {} in {}", got, full_count));
            ctx.set_cur(counting_status_idx, got);
            ctx.set_max(counting_status_idx, full_count);
            ctx.set_msg(whats_now_status_idx, &format!("waiting response..."));
            let request_start = Instant::now();
            let response = ctx.client.get(&endpoint)
                .query(&[("_offset", format!("{}", got))])
                .send()
                .await
                .unwrap()
                ;
            let url = response.url().clone();
            match response.status() {
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
                    ctx.set_msg(whats_now_status_idx, &format!("known 5xx status so wait for 5 minutes"));
                    tokio::time::delay_for(Duration::minutes(5).to_std().unwrap()).await;
                    continue
                }
                | StatusCode::OK
                => {
                    ctx.set_msg(whats_now_status_idx, &format!("reading data..."));
                    let bytes = response.bytes()
                        .await
                        .unwrap();
                    let request_end = Instant::now();

                    ctx.set_request_time(Duration::from_std(request_end - request_start).unwrap());

                    ctx.set_msg(whats_now_status_idx, &format!("writing to file..."));
                    // write to file
                    let path = dir.join(format!("ranking_{:06}.json", got));
                    match tokio::fs::write(&path, &bytes).await {
                        Ok(_) => {}
                        Err(_) => ctx.add_err(&format!("cannot write file {}: {}", &url, path.display())),
                    }
                    ctx.set_msg(whats_now_status_idx, &format!("parsing json..."));
                    // parse and set variables
                    let finished = match process_json(&bytes, &mut got, &mut full_count) {
                        None => {
                            ctx.add_err(&format!("cannot parse file {}: {}", &url, path.display()));
                            false
                        }
                        Some(finished) => finished
                    };

                    ctx.set_msg(whats_now_status_idx, &format!("waiting for server load reduction..."));
                    tokio::time::delay_until(ctx.get_wait_until(request_start).into())
                        .await;
                    if finished {
                        break;
                    }
                }
                _ => {
                    ctx.add_err(&format!("unknown response code getting {}: {}", &url, response.status()));

                    ctx.set_msg(whats_now_status_idx, &format!("unknown status so wait for 5 minutes"));
                    tokio::time::delay_for(Duration::minutes(5).to_std().unwrap()).await;
                }
            }
        }

        ctx.set_msg(whats_now_status_idx, &format!("getting version after get..."));
        let post_version = get_snapshot_version(ctx, &dir).await;
        if *pre_version != *post_version {
            ctx.add_info(&format!("version was changed at the end: {}", since));
            continue
        } else {
            break
        }
    }

    ctx.pop_status(whats_now_status_idx);
    ctx.pop_status(counting_status_idx);
}

async fn get_snapshot_version(ctx: &mut Context<'_>, dir: &std::path::PathBuf) -> Bytes
{
    let whats_now_status_idx = ctx.add_status("preprocessing");
    ctx.set_max(whats_now_status_idx, u32::max_value());

    loop {
        ctx.set_msg(whats_now_status_idx, &format!("waiting response..."));
        let request_start = Instant::now();
        let response = ctx.client.get(SEARCH_SNAPSHOT_V2_VERSION)
            .send()
            .await
            .unwrap();
        let url = response.url().clone();
        match response.status() {
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
                ctx.set_msg(whats_now_status_idx, &format!("known 5xx status so wait for 1 minutes"));
                tokio::time::delay_for(Duration::minutes(1).to_std().unwrap()).await;
                continue
            }
            StatusCode::OK => {
                ctx.set_msg(whats_now_status_idx, &format!("reading data..."));
                let bytes = response.bytes()
                    .await
                    .unwrap();
                let request_end = Instant::now();

                ctx.set_request_time(Duration::from_std(request_end - request_start).unwrap());

                ctx.set_msg(whats_now_status_idx, &format!("writing to file..."));
                // write to file
                let path = dir.join("version.json");
                match tokio::fs::write(&path, &bytes).await {
                    Ok(_) => {}
                    Err(_) => ctx.add_err(&format!("cannot write file {}: {}", &url, path.display())),
                }

                ctx.pop_status(whats_now_status_idx);
                return bytes;
            }
            _ => {
                ctx.add_err(&format!("unknown response code getting {}: {}", &url, response.status()));
            }
        }
    }
}

fn process_json(
    bytes: &bytes::Bytes,
    got: &mut u32,
    full_count: &mut u32,
) -> Option<bool> {
    let json_str = std::str::from_utf8(bytes.as_ref()).ok()?;
    let json: Map<String, JValue> = serde_json::from_str(json_str).ok()?;
    let meta: &Map<String, JValue> = json.get("meta")?.as_object()?;
    let total_count = meta.get("totalCount")?.as_i64()?;
    let data = json.get("data")?.as_array()?;

    *full_count = total_count as u32;
    *got += data.len() as u32;

    Some(data.len() == 0)
}
