use std::env;
use std::process::exit;

pub(crate) fn parse_options() -> Options {
    let args: Vec<_> = env::args().collect();
    if args.len() != 3 {
        eprintln!("{} <input bin> <output dir>", &args[0]);
        exit(-1);
    }
    let input_bin = args[1].to_string();
    let output_html = args[2].to_string();

    Options {
        input_bin,
        output_dir: output_html,
    }
}

pub struct Options {
    pub input_bin: String,
    pub output_dir: String,
}
