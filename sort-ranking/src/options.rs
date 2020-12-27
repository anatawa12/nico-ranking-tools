use std::env;
use std::process::exit;

pub(crate) fn parse_options() -> Options {
    let args: Vec<_> = env::args().collect();
    if args.len() != 4 {
        eprintln!("{} <input bin> <output bin> <ranking-type>", &args[0]);
        exit(-1);
    }
    let input_bin = args[1].to_string();
    let output_bin = args[2].to_string();
    let ranking_type = &args[3];

    let ranking_type = match ranking_type.as_str() {
        "watch-sum" => RankingType::WatchSum,
        "watch-cnt" => RankingType::WatchCnt,
        "watch-lng" => RankingType::WatchLng,
        _ => {
            eprintln!("invlaid ranking-type. must be either watch-sum, watch-cnt, or watch-lng: {}", ranking_type);
            exit(-1);
        },
    };

    Options {
        input_bin,
        output_bin,
        ranking_type,
    }
}

pub struct Options {
    pub input_bin: String,
    pub output_bin: String,
    pub ranking_type: RankingType,
}

pub enum RankingType {
    WatchSum,
    WatchCnt,
    WatchLng,
}
