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
    pub description	: String,
    pub view_counter: u32,
    pub mylist_counter: u32,
    pub length_seconds: std::time::Duration,
    pub thumbnail_url: String,
    pub start_time: DateTime<FixedOffset>,
    pub last_res_body: String,
    pub comment_counter: u32,
    pub last_comment_time: DateTime<FixedOffset>,
    pub category_tags: String,
    pub tags: Vec<String>,
    pub genre: String,
    pub genre_keyword: String,
}
