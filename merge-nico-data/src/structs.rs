use serde::{Serialize, Deserialize};
use chrono::{DateTime, FixedOffset};

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub(crate) struct VersionJson {
    pub(crate) last_modified: DateTime<FixedOffset>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub(crate) struct RankingJson {
    pub(crate) meta: RankingMeta,
    pub(crate) data: Vec<RankingVideoData>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub(crate) struct RankingMeta {
    #[serde(rename = "totalCount")]
    pub(crate) total_count: u64,
    pub(crate) last_modified: Option<DateTime<FixedOffset>>,
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub(crate) struct RankingVideoData {
    #[serde(rename = "contentId")]
    pub(crate) content_id: String,
    #[serde(rename = "lengthSeconds")]
    pub(crate) length_seconds: u32,
    #[serde(rename = "viewCounter")]
    pub(crate) view_counter: u32,
    #[serde(rename = "startTime")]
    pub(crate) start_time: DateTime<FixedOffset>,
}
