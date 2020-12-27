use std::env;
use std::process::exit;

pub(crate) fn parse_options() -> Options {
    let mut args = env::args();
    if args.len() < 3 {
        eprintln!("{} <out csv> <ranking bin files...>", &args.nth(0).unwrap());
        exit(-1);
    }
    let out_csv = args.nth(1).unwrap();
    let ranking_bins: Vec<String> = args.collect();

    Options {
        out_csv,
        ranking_bins,
    }
}

pub struct Options {
    pub out_csv: String,
    pub ranking_bins: Vec<String>,
}
