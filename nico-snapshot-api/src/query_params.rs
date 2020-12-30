use super::FilterJson;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::export::Formatter;
use serde::de::Unexpected;
use serde::export::fmt::Display;
use std::str::FromStr;
use super::response::ResponseJson;

#[derive(Serialize, Eq, PartialEq, Debug, Clone)]
pub struct QueryParams {
    q: String,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default="Vec::new")]
    #[serde(with="comma_string_vec")]
    targets: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default="Vec::new")]
    #[serde(with="comma_string_vec")]
    fields: Vec<String>,
    #[serde(with="string_json")]
    #[serde(rename="jsonFilter")]
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default="Option::None")]
    json_filter: Option<FilterJson>,
    #[serde(rename="_sort")]
    sort: SortingWithOrder,
    #[serde(rename="_offset")]
    #[serde(skip_serializing_if="is_zero")]
    offset: u32,
    #[serde(rename="_limit")]
    #[serde(skip_serializing_if="is_ten")]
    limit: u32,
    #[serde(rename="_context")]
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(default="Option::None")]
    context: Option<String>,
}

impl QueryParams {
    pub fn new(query: &str, sorting: SortingWithOrder) -> QueryParams {
        QueryParams {
            q: query.to_owned(),
            targets: Vec::new(),
            fields: Vec::new(),
            json_filter: None,
            sort: sorting,
            offset: 0,
            limit: 10,
            context: None,
        }
    }

    pub fn set_query(&mut self, query: String) {
        self.q = query;
    }

    pub fn with_targets(&mut self, args: &[&str]) {
        self.targets.append(&mut args.iter().map(|x| (x as &str).to_owned()).collect());
    }

    pub fn with_fields(&mut self, args: &[&str]) {
        self.fields.append(&mut args.iter().map(|x| (x as &str).to_owned()).collect());
    }

    pub fn set_filter(&mut self, filter: FilterJson) {
        self.json_filter = Some(filter);
    }

    pub fn set_sort(&mut self, sorting: SortingWithOrder) {
        self.sort = sorting;
    }

    pub fn set_offset(&mut self, offset: u32) {
        if !(0..=100_000).contains(&offset) {
            panic!("limit out of range. must be in 0..=100_000")
        }
        self.offset = offset;
    }

    pub fn set_limit(&mut self, limit: u32) {
        if !(0..=100).contains(&limit) {
            panic!("limit out of range. must be in 0..=100")
        }
        self.limit = limit;
    }

    pub fn set_context(&mut self, context: &str) {
        if context.len() > 40 {
            panic!("context too long")
        }
        self.context = Some(context.to_owned());
    }

    pub async fn get(&self, client: &reqwest::Client) -> reqwest::Result<ResponseJson> {
        const SEARCH_SNAPSHOT_V2_ENDPOINT: &str = "https://api.search.nicovideo.jp/api/v2/snapshot/video/contents/search";
        Ok(client.get(SEARCH_SNAPSHOT_V2_ENDPOINT)
            .query(&self)
            .send()
            .await?
            .json()
            .await?)
    }
}

fn is_zero(v: &u32) -> bool {
    *v == 0
}

fn is_ten(v: &u32) -> bool {
    *v == 10
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum SortingWithOrder {
    Decreasing(RankingSorting),
    Increasing(RankingSorting),
}

impl Serialize for SortingWithOrder {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {
        serializer.serialize_str(
            &match self {
                SortingWithOrder::Decreasing(sort) => format!("-{}", sort),
                SortingWithOrder::Increasing(sort) => format!("+{}", sort),
            }
        )
    }
}

impl <'de> Deserialize<'de> for SortingWithOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        use serde::de::Visitor;
        use serde::de::Expected;
        struct VisitorImpl;
        impl <'de> Visitor<'de> for VisitorImpl {
            type Value = SortingWithOrder;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "+ or - for order then Sorting")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
                if v.len() <= 1 {
                    return Err(E::invalid_value(Unexpected::Str(v), &self))
                }
                enum Order {
                    Decreasing, Increasing,
                }
                let order = match v.as_bytes()[0] {
                    b'-' => Order::Decreasing,
                    b'+' => Order::Increasing,
                    _ => return Err(E::invalid_value(Unexpected::Str(v), &"+ or - expected"))
                };
                if let Some(val) = RankingSorting::from_str(&v[1..]) {
                    match order {
                        Order::Decreasing => Ok(SortingWithOrder::Decreasing(val)),
                        Order::Increasing => Ok(SortingWithOrder::Increasing(val)),
                    }
                } else {
                    Err(E::invalid_value(Unexpected::Str(v), &"sorting expected"))
                }
            }
        }
        deserializer.deserialize_str(VisitorImpl)
    }
}

macro_rules! ranking_sorting {
    ( $( $name:ident ( $str: expr ) ),+ $(,)? ) => {
        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        pub enum RankingSorting {
            $(
                $name,
            )*
        }

        impl RankingSorting {
            pub fn to_str(&self) -> &'static str {
                match self {
                    $(
                        RankingSorting::$name => $str,
                    )*
                }
            }

            pub fn from_str(str: &str) -> Option<RankingSorting> {
                match str {
                    $(
                        $str => Some(RankingSorting::$name),
                    )*
                    _ => None
                }
            }

            fn expecting_string() -> &'static str {
                concat!("one of ",
                    $(
                        $str, ", ",
                    )*
                )
            }
        }
    };
}

ranking_sorting! {
    ViewCounter("viewCounter"),
    MylistCounter("mylistCounter"),
    LengthSeconds("lengthSeconds"),
    StartTime("startTime"),
    CommentCounter("commentCounter"),
    LastCommentTime("lastCommentTime"),
}

impl RankingSorting {
    pub fn decreasing(self) -> SortingWithOrder {
        SortingWithOrder::Decreasing(self)
    }

    pub fn increasing(self) -> SortingWithOrder {
        SortingWithOrder::Decreasing(self)
    }
}

pub struct RankingSortingFromStrError;

impl FromStr for RankingSorting {
    type Err = RankingSortingFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RankingSorting::from_str(s).ok_or(RankingSortingFromStrError)
    }
}

impl Display for RankingSorting {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_str())
    }
}

impl Serialize for RankingSorting {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
        S: Serializer {
        serializer.serialize_str(self.to_str())
    }
}
impl <'de> Deserialize<'de> for RankingSorting {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
        D: Deserializer<'de> {
        use serde::de::Visitor;
        struct VisitorImpl;
        impl <'de> Visitor<'de> for VisitorImpl {
            type Value = RankingSorting;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "{}", RankingSorting::expecting_string())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
                if let Some(val) = RankingSorting::from_str(v) {
                    Ok(val)
                } else {
                    Err(E::invalid_value(Unexpected::Str(v), &self))
                }
            }
        }
        deserializer.deserialize_str(VisitorImpl)
    }
}

mod comma_string_vec {
    use serde::{Serializer, Deserializer, Deserialize};

    pub(super) fn serialize<S>(vec: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let str = vec.join(",");
        serializer.serialize_str(&str)
    }

    #[allow(dead_code)]
    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error> where
        D: Deserializer<'de> {
        let str = <String as Deserialize>::deserialize(deserializer)?;
        Ok(str.split("").map(|str| str.to_string()).collect())
    }
}

mod string_json {
    use serde::{Serializer, Deserializer, Deserialize};
    use super::FilterJson;
    use serde::ser::Error as SerError;
    use serde::de::Error as DeError;
    use serde_json::Error;

    pub(super) fn serialize<S>(json: &Option<FilterJson>, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        match serde_json::to_string(json) {
            Ok(str) => Ok(serializer.serialize_str(&str)?),
            Err(err) => Err(S::Error::custom(err.to_string())),
        }
    }

    #[allow(dead_code)]
    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Option<FilterJson>, D::Error> where
        D: Deserializer<'de> {
        let str = <String as Deserialize>::deserialize(deserializer)?;
        match serde_json::from_str::<FilterJson>(&str) {
            Ok(json) => Ok(Some(json)),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}
