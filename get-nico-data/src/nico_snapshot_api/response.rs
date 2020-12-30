use serde_json::Value;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ResponseJson {
    pub meta: MetaObject,
    pub data: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct MetaObject {
    #[serde(with="status_code_serde")]
    pub status: reqwest::StatusCode,
    pub id: String,
    #[serde(rename="totalCount")]
    pub total_count: usize,
}

mod status_code_serde {
    use serde::{Serializer, Deserializer, Deserialize};
    use serde::de::{Error, Unexpected};
    use reqwest::StatusCode;

    pub(super) fn serialize<S>(code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        serializer.serialize_u16(code.as_u16())
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error> where
        D: Deserializer<'de> {
        let code = <u16 as Deserialize>::deserialize(deserializer)?;
        if let Ok(code) = StatusCode::from_u16(code) {
            Ok(code)
        } else {
            Err(D::Error::invalid_value(Unexpected::Unsigned(code as u64), &"100..=999"))
        }
    }
}
