mod filter_json;
mod query_params;
mod response;
mod serializers;

pub use filter_json::*;
pub use query_params::*;
pub use response::*;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use reqwest::Client;

const SEARCH_SNAPSHOT_V2_VERSION: &str = "https://api.search.nicovideo.jp/api/v2/snapshot/version";

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub struct SnapshotVersion {
    last_modified: DateTime<FixedOffset>
}

pub async fn snapshot_version(client: &Client) -> reqwest::Result<SnapshotVersion> {
    Ok(client.get(SEARCH_SNAPSHOT_V2_VERSION).send().await?.json().await?)
}
