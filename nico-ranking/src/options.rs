use clap::*;
use chrono::{FixedOffset, TimeZone, NaiveDate, Duration, DateTime};
use std::process::exit;
use nico_snapshot_api::FilterJson;
use std::io::BufReader;
use std::fs::File;
use std::cmp::Ordering;
use std::str::FromStr;
use std::result::Result;

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
        .arg(Arg::with_name("filter")
            // https://bit.ly/3aOXNn6: https://site.nicovideo.jp/search-api-docs/snapshot#＊4-jsonフィルタ指定仕様
            .help("path to filter json. see https://bit.ly/3aOXNn6")
            .takes_value(true)
            .short("-f")
            .long("--filter"))
        .arg(Arg::with_name("ranking_type")
            .help("type of ranking")
            .possible_values(&["watch-sum", "watch-cnt", "watch-lng"])
            .takes_value(true)
            .required(true)
            .short("-r")
            .long("--ranking-type"))
        .arg(Arg::with_name("phase_since")
            .help("this program starts from")
            .possible_values(Phase::FROM_STRINGS)
            .takes_value(true)
            .short("-p")
            .long("--phase"))
        ;
    let matches = app.get_matches();

    let since = matches.value_of("since")
        .map(|date| jst_timezone
            .from_local_date(&NaiveDate::parse_from_str(date, DATE_FORMAT_WITH_TIME)
                .unwrap_or_else(|err| exiting_errf!("since: {}", err)))
            .unwrap()
            .and_hms(0, 0, 0));

    let until = matches.value_of("until")
        .map(|date| jst_timezone
            .from_local_date(&NaiveDate::parse_from_str(date, DATE_FORMAT_WITH_TIME)
                .unwrap_or_else(|err| exiting_errf!("until: {}", err)))
            .unwrap()
            .and_hms(0, 0, 0));

    let duration = matches.value_of("duration")
        .map(|duration| Duration::from_std(parse_duration::parse(duration)
            .unwrap_or_else(|err| exiting_errf!("duration: {}", err))).unwrap());

    let filter = matches.value_of("filter")
        .map(|path| {
            let file = File::open(path).unwrap_or_else(|err| exiting_errf!("filter: {}", err));
            let file = BufReader::new(file);
            serde_json::from_reader::<_, FilterJson>(file)
                .unwrap_or_else(|err| exiting_errf!("filter: {}", err))
        });

    let ranking_type = matches.value_of("ranking_type").unwrap().to_string();

    let phase_since = matches.value_of("phase_since")
        .map_or(Phase::GetNicoData, |str| Phase::from_str(str).unwrap());

    Options {
        since,
        until,
        duration,
        filter,
        ranking_type,
        phase_since,
    }
}

pub struct Options {
    pub since: Option<DateTime<FixedOffset>>,
    pub until: Option<DateTime<FixedOffset>>,
    pub duration: Option<Duration>,
    pub filter: Option<FilterJson>,
    pub ranking_type: String,
    pub phase_since: Phase,
}

#[derive(PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Phase {
    GetNicoData,
    MergeNicoData,
    SortRanking,
    MergeRankings,
    HtmlGen,
}

impl Phase {
    const FROM_STRINGS: &'static [&'static str] = &[
        "get-nico-data",
        "merge-nico-data",
        "sort-ranking",
        "merge-rankings",
        "html-gen",
    ];

    fn index(self) -> u8 {
        match self {
            Phase::GetNicoData => 0,
            Phase::MergeNicoData => 1,
            Phase::SortRanking => 2,
            Phase::MergeRankings => 3,
            Phase::HtmlGen => 4,
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Phase::GetNicoData => "get-nico-data",
            Phase::MergeNicoData => "merge-nico-data",
            Phase::SortRanking => "sort-ranking",
            Phase::MergeRankings => "merge-rankings",
            Phase::HtmlGen => "html-gen",
        }
    }

    fn from_str(str: &str) -> Option<Phase> {
        match str {
            "get-nico-data" => Some(Phase::GetNicoData),
            "merge-nico-data" => Some(Phase::MergeNicoData),
            "sort-ranking" => Some(Phase::SortRanking),
            "merge-rankings" => Some(Phase::MergeRankings),
            "html-gen" => Some(Phase::HtmlGen),
            _  => None
        }
    }
}

pub struct PhaseFromStrError;

impl FromStr for Phase {
    type Err = PhaseFromStrError;

    fn from_str(s: &str) -> Result<Self, PhaseFromStrError> {
        Phase::from_str(s).ok_or(PhaseFromStrError)
    }
}

impl Ord for Phase {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index().cmp(&other.index())
    }
}

impl ToString for Phase {
    fn to_string(&self) -> String {
        self.to_str().to_string()
    }
}
