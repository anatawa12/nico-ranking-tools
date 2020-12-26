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
    #[serde(rename = "lengthSeconds")]
    pub length_seconds: u32,
    #[serde(rename = "viewCounter")]
    pub view_counter: u32,
    #[serde(rename = "startTime")]
    pub start_time: DateTime<FixedOffset>,
}
