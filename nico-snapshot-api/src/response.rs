use serde_json::Value;
use serde::{Serialize, Deserialize};
use super::serializers;

#[derive(Serialize, Deserialize)]
pub struct ResponseJson {
    pub meta: MetaObject,
    pub data: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct MetaObject {
    #[serde(with="serializers::status_code_serde")]
    pub status: reqwest::StatusCode,
    pub id: String,
    #[serde(rename="totalCount")]
    pub total_count: usize,
}
