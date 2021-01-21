use serde::{Serialize, Deserialize};
use super::serializers;
use std::time::Duration;
use chrono::{FixedOffset, DateTime};

#[derive(Serialize, Deserialize)]
pub struct ResponseJson {
    pub meta: MetaObject,
    pub data: Vec<VideoInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct MetaObject {
    #[serde(with="serializers::status_code_serde")]
    pub status: reqwest::StatusCode,
    pub id: String,
    #[serde(rename="totalCount")]
    pub total_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct VideoInfo {
    #[serde(rename="contentId")]
    pub content_id: Option<String>,
    pub title: Option<String>,
    pub description	: Option<String>,
    #[serde(rename="viewCounter")]
    pub view_counter	: Option<u32>,
    #[serde(rename="mylistCounter")]
    pub mylist_counter: Option<u32>,
    #[serde(with="serializers::duration_opt_seconds")]
    #[serde(rename="lengthSeconds")]
    pub length_seconds: Option<Duration>,
    #[serde(rename="thumbnailUrl")]
    pub thumbnail_url: Option<String>,
    #[serde(rename="startTime")]
    pub start_time: Option<DateTime<FixedOffset>>,
    #[serde(rename="lastResBody")]
    pub last_res_body: Option<String>,
    #[serde(rename="commentCounter")]
    pub comment_counter: Option<u32>,
    #[serde(rename="lastCommentTime")]
    pub last_comment_time: Option<DateTime<FixedOffset>>,
    #[serde(rename="categoryTags")]
    pub category_tags: Option<String>,
    #[serde(with="serializers::space_string_vec_opt")]
    pub tags: Option<Vec<String>>,
    pub genre: Option<String>,
}
