
pub(crate) mod comma_string_vec {
    use serde::{Serializer, Deserializer, Deserialize};

    pub(crate) fn serialize<S>(vec: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let str = vec.join(",");
        serializer.serialize_str(&str)
    }

    #[allow(dead_code)]
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error> where
        D: Deserializer<'de> {
        let str = <String as Deserialize>::deserialize(deserializer)?;
        Ok(str.split("").map(|str| str.to_string()).collect())
    }
}

pub(crate) mod status_code_serde {
    use serde::{Serializer, Deserializer, Deserialize};
    use serde::de::{Error, Unexpected};
    use reqwest::StatusCode;

    pub(crate) fn serialize<S>(code: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        serializer.serialize_u16(code.as_u16())
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<StatusCode, D::Error> where
        D: Deserializer<'de> {
        let code = <u16 as Deserialize>::deserialize(deserializer)?;
        if let Ok(code) = StatusCode::from_u16(code) {
            Ok(code)
        } else {
            Err(D::Error::invalid_value(Unexpected::Unsigned(code as u64), &"100..=999"))
        }
    }
}
