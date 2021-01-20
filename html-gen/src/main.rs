use crate::options::parse_options;
use crate::utils::MyIterUtil;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use itertools::Itertools;
use crate::ymd_print::ymd_to_string;
use crate::numeral_print::numeral_to_string;
use indicatif::{ProgressBar, ProgressStyle};
use crate::index_file::RankingPage;
use structs::NewVideoInfo;
use crate::progress_reader::ProgressReader;

mod options;
mod utils;
mod ymd_print;
mod numeral_print;
mod index_file;
mod progress_reader;

fn main() {
    let options = parse_options();

    let input_bin = File::open(&options.input_bin).unwrap();
    let input_bin = BufReader::new(input_bin);
    let input_bin_size = fs::metadata(&options.input_bin).unwrap().len();

    fs::create_dir_all(&options.output_dir).unwrap();

    let mut page_infos = Vec::<RankingPage>::new();
    let per_page: usize = 200;
    let mut page_number: u64 = 0;

    let progress = ProgressBar::new(input_bin_size);
    progress.set_message("reading binary...");
    progress.enable_steady_tick(10);
    set_style(&progress);
    let mut input_bin = ProgressReader::new(&progress, input_bin);

    let list: Vec<NewVideoInfo> = bincode::deserialize_from(&mut input_bin).unwrap();

    progress.finish();
    drop(progress);

    let progress = ProgressBar::new(list.len() as u64);
    progress.enable_steady_tick(10);
    set_style(&progress);

    for (index, (elements, has_next)) in list.iter()
        .enumerate()
        .chunks(per_page)
        .into_iter()
        .with_has_next()
        .enumerate() {
        progress.set_message(&format!("page #{}", page_number));
        let versions: Vec<_> = elements.collect();
        let last_page_count = versions.len() as u64;
        let info = PageInfo {
            per_page: per_page as u64,
            page_number,
            has_next,
            page_count: last_page_count,
        };
        let cnt = process_a_chunk(versions, &options.output_dir, &info).unwrap();
        progress.inc(1);
        page_number += 1;
        page_infos.append(&mut vec![RankingPage{
            index: index as u64,
            first_rank: info.start_rank(),
            last_rank: info.start_rank() - 1 + last_page_count,
        }]);
        if cnt != last_page_count {
            break
        }
    }
    progress.finish_with_message("finished");

    // write index file
    index_file::index_file(&options.output_dir, &page_infos).unwrap();
}

fn process_a_chunk<'a, Itr>(versions: Itr, output_dir: &String, info: &PageInfo) -> std::io::Result<u64>
    where Itr : IntoIterator<Item = (usize, &'a NewVideoInfo)> {
    let output_html = format!("{}/ranking-{}.html", output_dir, info.page_number);
    let output_html = File::create(output_html)?;
    let mut output_html = BufWriter::new(output_html);
    let mut count = 0;

    write_heading(&mut output_html, info)?;
    writeln!(&mut output_html, r#"<ul class="container">"#)?;

    for (rank, version) in versions {
        let sum_dur = version.view_counter * version.length_seconds;
        let video_id = &version.content_id;
        let view_count = version.view_counter as u64;
        let video_length = version.length_seconds;

        writeln!(&mut output_html, r#"    <li class="grid-item">"#)?;
        writeln!(&mut output_html, r#"        <div class="ranking-header"><a href="https://nicovideo.jp/watch/{}" class="ranking-header-link">#{}</a></div>"#,
                 video_id, rank)?;
        writeln!(&mut output_html, r#"        <div>"#)?;
        writeln!(&mut output_html, r#"            <div>{}</div>"#, ymd_to_string(sum_dur))?;
        writeln!(&mut output_html, r#"            <div>{} {}回再生</div>"#, ymd_to_string(video_length), numeral_to_string(view_count))?;
        writeln!(&mut output_html, r#"            <iframe class="nico-frame lazy" width="312" height="176" scrolling="no" data-src="https://ext.nicovideo.jp/thumb/{}"></iframe>"#,
                 video_id)?;
        writeln!(&mut output_html, r#"        </div>"#)?;
        writeln!(&mut output_html, r#"    </li>"#)?;
        count += 1;
    }

    writeln!(&mut output_html, r#"</ul>"#)?;

    write_footing(&mut output_html, info)?;
    Ok(count)
}

fn write_heading<W: Write>(output_html: &mut W, info: &PageInfo) -> std::io::Result<()> {
    let (start_rank, last_rank) = info.page_rank_range();

    writeln!(output_html, r#"<!DOCTYPE html>"#)?;
    writeln!(output_html, r#"<html lang="en">"#)?;
    writeln!(output_html, r#"<head>"#)?;
    writeln!(output_html, "{}", include_str!("template.head.html")
        .replace("{title}", &format!("人類が動画にかけた時間のランキング({}位〜{}位)", start_rank, last_rank)))?;
    writeln!(output_html, r#"</head>"#)?;
    writeln!(output_html, r#"<body>"#)?;
    writeln!(output_html, "{}", include_str!("template.body.head.html"))?;
    writeln!(output_html, r#"<header class="header">"#)?;
    write_prev_cur_next(output_html, info)?;
    writeln!(output_html, r#"</header>"#)?;
    Ok(())
}

fn write_footing<W: Write>(output_html: &mut W, info: &PageInfo) -> std::io::Result<()> {
    writeln!(output_html, r#"<footer class="footer">"#)?;
    write_prev_cur_next(output_html, info)?;
    writeln!(output_html, r#"</footer>"#)?;
    writeln!(output_html, "{}", include_str!("template.body.foot.html"))?;
    writeln!(output_html, r#"</body>"#)?;
    writeln!(output_html, r#"</html>"#)?;
    Ok(())
}

fn write_prev_cur_next<W: Write>(output_html: &mut W, info: &PageInfo) -> std::io::Result<()> {
    let (start_rank, last_rank) = info.page_rank_range();

    match info.prev_rank_range() {
        None => {
            writeln!(output_html, r#"    <a href="index.html" class="left">← prev (ランキングトップ)</a>"#, )?;
        }
        Some((since, last)) => {
            writeln!(output_html, r#"    <a href="ranking-{}.html" class="left">← prev ({}位〜{}位)</a>"#,
                     info.page_number - 1, since, last)?;
        }
    }
    match info.next_rank_range() {
        None => {
            writeln!(output_html, r#"    <a href="index.html" class="right">(ランキングトップ) next →</a>"#)?;
        }
        Some((since, last)) => {
            writeln!(output_html, r#"    <a href="ranking-{}.html" class="right">({}位〜{}位) next →</a>"#,
                     info.page_number + 1, since, last)?;
        }
    }
    writeln!(output_html, r#"    <div class="center">{}位〜{}位</div>"#, start_rank, last_rank)?;
    Ok(())
}

fn set_style(progress: &ProgressBar) {
    progress.set_style(ProgressStyle::default_bar().template("[{elapsed_precise}] {bar:40blue} {pos:>7}/{len:7} ({percent}%) {msg}")
        .progress_chars("##-"));
}

pub struct PageInfo {
    per_page: u64,
    // starts with 0
    page_number: u64,
    page_count: u64,
    has_next: bool,
}

impl PageInfo {
    fn prev_info(&self) -> Option<PageInfo> {
        if self.page_number == 0 {
            None
        } else {
            Some(PageInfo {
                per_page: self.per_page,
                page_number: self.page_number - 1,
                has_next: true,
                page_count: self.per_page,
            })
        }
    }

    fn next_info(&self) -> Option<PageInfo> {
        if !self.has_next {
            None
        } else {
            Some(PageInfo {
                per_page: self.per_page,
                page_number: self.page_number + 1,
                has_next: false, //unknown
                page_count: self.page_count, // unknown
            })
        }
    }

    pub fn prev_rank_range(&self) -> Option<(u64, u64)> {
        self.prev_info().map(|inf| inf.page_rank_range())
    }

    pub fn next_rank_range(&self) -> Option<(u64, u64)> {
        self.next_info().map(|inf| inf.page_rank_range())
    }

    pub fn page_rank_range(&self) -> (u64, u64) {
        (self.start_rank(), self.last_rank())
    }

    pub fn start_rank(&self) -> u64 {
        self.per_page * self.page_number + 1
    }

    pub fn last_rank(&self) -> u64 {
        self.start_rank() + self.page_count - 1
    }
}
