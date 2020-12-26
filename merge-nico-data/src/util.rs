use std::path::{PathBuf, Path};
use regex::Regex;
use std::fs;

pub fn sorted_ls_matches_regex<P: AsRef<Path>>(
    base: P,
    regex: &Regex,
) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(base)
        .unwrap()
        .map(|x| x.unwrap().path())
        .filter(|path| regex.is_match(path.file_name().unwrap().to_str().unwrap()))
        .collect();

    files.sort_by(|a, b| {
        a.file_name().unwrap().to_str().unwrap()
            .cmp(b.file_name().unwrap().to_str().unwrap())
    });

    return files
}
