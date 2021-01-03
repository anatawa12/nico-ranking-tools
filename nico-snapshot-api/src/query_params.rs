use super::FilterJson;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::export::Formatter;
use serde::de::Unexpected;
use serde::export::fmt::Display;
use std::str::FromStr;
use super::response::ResponseJson;
use super::serializers;

#[derive(Serialize, Eq, PartialEq, Debug, Clone)]
pub struct QueryParams {
    q: String,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default="Vec::new")]
    #[serde(with="serializers::comma_string_vec")]
    targets: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default="Vec::new")]
    #[serde(with="serializers::comma_string_vec")]
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
            .error_for_status()?
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

macro_rules! string_enum {
    ( $type_name: ident, $value_name_str: expr, $error_name: ident : $( $name:ident ( $str: expr ) ),+ $(,)? ) => {
        #[derive(Eq, PartialEq, Debug, Copy, Clone)]
        pub enum $type_name {
            $(
                $name,
            )*
        }

        impl $type_name {
            pub fn to_str(&self) -> &'static str {
                match self {
                    $(
                        $type_name::$name => $str,
                    )*
                }
            }

            pub fn from_str(str: &str) -> Option<$type_name> {
                match str {
                    $(
                        $str => Some($type_name::$name),
                    )*
                    _ => None
                }
            }
        }

        impl FromStr for $type_name {
            type Err = $error_name;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                $type_name::from_str(s).ok_or($error_name { value: s.to_owned() })
            }
        }

        impl Display for $type_name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.to_str())
            }
        }

        pub struct $error_name {
            value: String,
        }

        impl Display for $error_name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(concat!("expected ", $value_name_str, " but was "))?;
                f.write_str(&self.value)?;
                Ok(())
            }
        }

        impl Serialize for $type_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where
                S: Serializer {
                serializer.serialize_str(self.to_str())
            }
        }

        impl <'de> Deserialize<'de> for $type_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where
                D: Deserializer<'de> {
                use serde::de::Visitor;
                struct VisitorImpl;
                impl <'de> Visitor<'de> for VisitorImpl {
                    type Value = $type_name;

                    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                        write!(formatter, $value_name_str)
                    }

                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
                        if let Some(val) = $type_name::from_str(v) {
                            Ok(val)
                        } else {
                            Err(E::invalid_value(Unexpected::Str(v), &self))
                        }
                    }
                }
                deserializer.deserialize_str(VisitorImpl)
            }
        }
   };
}

string_enum! {
    RankingSorting, "ranking sorting name", RankingSortingFromStrError:
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

mod string_json {
    use serde::{Serializer, Deserializer, Deserialize};
    use super::FilterJson;
    use serde::ser::Error as SerError;
    use serde::de::Error as DeError;

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
