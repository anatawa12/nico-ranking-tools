use clap::*;
use chrono::{FixedOffset, TimeZone, NaiveDate, Duration, DateTime};
use std::process::exit;

macro_rules! exiting_errf {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        exit(-1)
    })
}

const DATE_FORMAT_WITH_TIME: &str = "%Y/%m/%d";

pub fn parse_options() -> Options {
    let jst_timezone = FixedOffset::east(9 * 3600);
    let app = app_from_crate!()
        .arg(Arg::with_name("since")
            .help("the begin date of find range. defaults the date starts SMILEVIDEO, 2020/03/06")
            .takes_value(true)
            .short("-s")
            .long("--since"))
        .arg(Arg::with_name("until")
            .help("the last date of find range. defaults now")
            .takes_value(true)
            .short("-u")
            .long("--until"))
        .arg(Arg::with_name("duration")
            .help("duration to be got at a time. defaults 1 week")
            .takes_value(true)
            .short("-d")
            .long("--duration"))
        .arg(Arg::with_name("out-to")
            .help("file to write to. defaults stdout")
            .takes_value(true)
            .short("-o")
            .long("--out"))
        .arg(Arg::with_name("contents-id-out")
            .help("file to write contents id proceed.")
            .takes_value(true)
            .short("-c")
            .long("--content-id-out"))
        ;
    let matches = app.get_matches();

    let since = matches.value_of("since")
        .map(|date| jst_timezone
            .from_local_date(&NaiveDate::parse_from_str(date, DATE_FORMAT_WITH_TIME)
                .unwrap_or_else(|err| exiting_errf!("since: {}", err)))
            .unwrap()
            .and_hms(0, 0, 0))
        .unwrap_or_else(|| jst_timezone.ymd(2007, 03, 06).and_hms(0, 0, 0));

    let until = matches.value_of("until")
        .map(|date| jst_timezone
            .from_local_date(&NaiveDate::parse_from_str(date, DATE_FORMAT_WITH_TIME)
                .unwrap_or_else(|err| exiting_errf!("until: {}", err)))
            .unwrap()
            .and_hms(0, 0, 0));

    let duration = matches.value_of("duration")
        .map(|duration| Duration::from_std(parse_duration::parse(duration)
            .unwrap_or_else(|err| exiting_errf!("duration: {}", err))).unwrap())
        .unwrap_or_else(|| Duration::weeks(1));

    let out = matches.value_of("out-to").map(|x| x.to_owned());

    let contents_id_out = matches.value_of("contents-id-out").map(|x| x.to_owned());

    Options {
        since,
        until,
        duration,
        out,
        contents_id_out
    }
}

pub struct Options {
    pub since: DateTime<FixedOffset>,
    pub until: Option<DateTime<FixedOffset>>,
    pub duration: Duration,
    pub out: Option<String>,
    pub contents_id_out: Option<String>,
}
