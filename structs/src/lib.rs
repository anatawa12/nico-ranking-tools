use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct VersionJson {
    pub last_modified: DateTime<FixedOffset>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct RankingJson {
    pub meta: RankingMeta,
    pub data: Vec<RankingVideoData>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct RankingMeta {
    #[serde(rename = "totalCount")]
    pub total_count: u64,
    pub last_modified: Option<DateTime<FixedOffset>>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct RankingVideoData {
    #[serde(rename = "contentId")]
    pub content_id: String,
    #[serde(default = "u32::max_value")]
    pub length_seconds: u32,
    #[serde(default = "u32::max_value")]
    pub view_counter: u32,
    #[serde(rename = "startTime")]
    pub start_time: DateTime<FixedOffset>,
}

fn is_max_value_u32(v: &u32) -> bool {
    *v == u32::max_value()
}

fn is_max_value_u64(v: &u64) -> bool {
    *v == u64::max_value()
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct RankingBin {
    pub meta: RankingMeta,
    pub data: Vec<RankingVideoDataBin>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct RankingVideoDataBin {
    #[serde(rename = "contentId")]
    pub content_id: String,
    #[serde(rename = "lengthSeconds")]
    pub length_seconds: u32,
    #[serde(rename = "viewCounter")]
    pub view_counter: u32,
    #[serde(rename = "startTime")]
    pub start_time: DateTime<FixedOffset>,
    pub ranking_counter: u64,
}

#[derive(Serialize, Deserialize)]
pub struct NewVideoInfo {
    pub last_modified: DateTime<FixedOffset>,
    pub content_id: String,
    pub title: String,
    pub description	: Option<String>,
    pub view_counter: u32,
    pub mylist_counter: u32,
    pub length_seconds: std::time::Duration,
    pub thumbnail_url: Option<String>,
    pub start_time: DateTime<FixedOffset>,
    pub last_res_body: Option<String>,
    pub comment_counter: u32,
    pub last_comment_time: Option<DateTime<FixedOffset>>,
    pub category_tags: Option<String>,
    pub tags: Vec<String>,
    pub genre: Option<String>,
}

#[cfg(feature="bincode")]
pub mod bincode_impl {
    use super::*;
    use bincode::Options;
    use serde::Deserialize;
    use bincode::{ErrorKind, Deserializer, DefaultOptions};
    use bincode::config::*;
    use bincode::de::read::IoReader;
    use std::io::{Read, Write};
    use std::borrow::Borrow;

    pub struct ReadIter<R> {
        de: Option<Deserializer<IoReader<R>, WithOtherTrailing<WithOtherIntEncoding<DefaultOptions, FixintEncoding>, AllowTrailing>>>,
        max: usize,
    }

    impl <R: Read> Iterator for ReadIter<R> {
        type Item = NewVideoInfo;

        fn next(&mut self) -> Option<Self::Item> {
            if let Some(de) = &mut self.de {
                loop {
                    match NewVideoInfo::deserialize(&mut *de) {
                        Ok(data) => {
                            // filter here
                            if data.tags.iter().any(|x| x == "作業用BGM") {
                                continue
                            }
                            return Some(data);
                        }
                        Err(err) => {
                            match err.borrow() {
                                ErrorKind::Io(err) => {
                                    match err.kind() {
                                        std::io::ErrorKind::UnexpectedEof => break,
                                        _ => Err(err).unwrap()
                                    }
                                }
                                err => Err(err).unwrap()
                            }
                        }
                    }
                }
            }
            self.de = None;
            None
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (0, Some(self.max))
        }
    }

    #[allow(dead_code)]
    pub fn read_file<R: Read>(reader: &mut R, size: usize) -> ReadIter<&mut R> {
        ReadIter {
            de: Some(Deserializer::with_reader(reader, DefaultOptions::new()
                .with_fixint_encoding()
                .allow_trailing_bytes())),
            max: size / 128,
        }
    }

    #[allow(dead_code)]
    pub fn write_file<'a, I, W>(data: I, writer: &mut W)
        where W: Write, I: Iterator<Item=&'a NewVideoInfo> {
        for value in data {
            bincode::serialize_into(&mut *writer, value).unwrap();
        }
    }
}
