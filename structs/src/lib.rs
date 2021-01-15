use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset, Utc};

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
    pub last_modified: DateTime<Utc>,
    pub content_id: String,
    pub title: String,
    pub description	: Option<String>,
    pub view_counter: u32,
    pub mylist_counter: u32,
    pub length_seconds: std::time::Duration,
    pub thumbnail_url: Option<String>,
    pub start_time: DateTime<Utc>,
    pub last_res_body: Option<String>,
    pub comment_counter: u32,
    pub last_comment_time: Option<DateTime<Utc>>,
    pub category_tags: Option<String>,
    pub tags: Vec<String>,
    pub genre: Option<String>,
}
