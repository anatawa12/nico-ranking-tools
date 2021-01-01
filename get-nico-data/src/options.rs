use clap::*;
use chrono::{FixedOffset, TimeZone, Date, NaiveDate, Duration, DateTime};
use std::process::exit;
use nico_snapshot_api::FilterJson;
use std::io::BufReader;
use std::fs::File;

macro_rules! exiting_errf {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        exit(-1)
    })
}

const DATE_FORMAT_WITH_TIME: &str = "%Y/%m/%d";
const ZERO_O_CLOCK_POSTFIX: &str = " 00:00:00";

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
        .arg(Arg::with_name("filter")
            // https://bit.ly/3aOXNn6: https://site.nicovideo.jp/search-api-docs/snapshot#＊4-jsonフィルタ指定仕様
            .help("path to filter json. see https://bit.ly/3aOXNn6")
            .takes_value(true)
            .short("-f")
            .long("--filter"))
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

    let filter = matches.value_of("filter")
        .map(|path| {
            let file = File::open(path).unwrap_or_else(|err| exiting_errf!("filter: {}", err));
            let file = BufReader::new(file);
            serde_json::from_reader::<_, FilterJson>(file)
                .unwrap_or_else(|err| exiting_errf!("filter: {}", err))
        });

    Options {
        since,
        until,
        duration,
        filter,
    }
}

pub struct Options {
    pub since: DateTime<FixedOffset>,
    pub until: Option<DateTime<FixedOffset>>,
    pub duration: Duration,
    pub filter: Option<FilterJson>,
}
