use chrono::{DateTime, FixedOffset};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use std::ops::Not;
use serde::ser::SerializeMap;

//
// [Serialize] and [Deserialize] are implemented manually
//
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum FilterJson {
    Equal(EqualFilter),
    Range(RangeFilter),
    Or(Vec<Self>),
    And(Vec<Self>),
    Not(Box<FilterJson>),
}

macro_rules! filters {
    (
        {
            $( $child_name0:ident ( $field_name0:expr, $child_name_snake0:ident, $type0:ty ) ),* ,
        },
        {
            $( $child_name1:ident ( $field_name1:expr, $child_name_snake1:ident, $type1:ty ) ),* ,
        }
    ) => {
        #[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
        #[serde(tag = "field", content = "value")]
        pub enum EqualFilter {
            $(
                #[serde(rename = $field_name0)]
                $child_name0($type0),
            )*
            $(
                #[serde(rename = $field_name1)]
                $child_name1($type1),
            )*
        }
        #[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
        #[serde(tag = "field")]
        pub enum RangeFilter {
            $(
                #[serde(rename = $field_name0)]
                $child_name0 {
                    from: $type0,
                    to: $type0,
                    #[serde(skip_serializing_if="filter_json_serde::bool_not")]
                    #[serde(default="filter_json_serde::bool_false")]
                    include_lower: bool,
                    #[serde(skip_serializing_if="filter_json_serde::bool_not")]
                    #[serde(default="filter_json_serde::bool_false")]
                    include_upper: bool,
                },
            )*
        }

        impl RangeFilter {
            $(
                pub fn $child_name_snake0(from: $type0, to: $type0) -> RangeFilter {
                    RangeFilter::$child_name0 {
                        from,
                        to,
                        include_lower: false,
                        include_upper: false
                    }
                }
            )*

            pub fn include_lower(&mut self) -> &mut Self {
                match self {
                    $(
                        RangeFilter::$child_name0{ include_lower: include_lower, .. } => *include_lower = true
                    ),*
                }
                self
            }

            pub fn include_upper(&mut self) -> &mut Self {
                match self {
                    $(
                        RangeFilter::$child_name0{ include_upper: include_upper, .. } => *include_upper = true
                    ),*
                }
                self
            }
        }
    };
}

filters! {
    {
        ViewCounter ("viewCounter", view_counter, u64),
        MyListCounter ("mylistCounter", mylist_counter, u64),
        LengthSeconds ("lengthSeconds", length_seconds, u64),
        StartTime ("startTime", start_time, DateTime<FixedOffset>),
        CommentCounter ("commentCounter", comment_counter, u64),
        LastCommentTime ("lastCommentTime", last_comment_time, DateTime<FixedOffset>),
    },
    {
        CategoryTags ("categoryTags", category_tags, String),
        Tags ("tags", tags, String),
        Genre ("genre", genre, String),
        GenreKeyword ("genreKeyword", genre_keyword, String),
    }
}

// serde serializing helpers
mod filter_json_serde {
    use serde::{Serialize, Serializer, Deserialize, Deserializer};
    use super::{FilterJson, EqualFilter, RangeFilter};
    use serde::ser::SerializeMap;

    #[derive(Serialize)]
    #[serde(tag = "type")]
    enum FilterJsonForSerialize<'a> {
        #[serde(rename = "equal")]
        Equal(&'a EqualFilter),
        #[serde(rename = "range")]
        Range(&'a RangeFilter),
        #[serde(rename = "or")]
        Or { filters: &'a Vec<FilterJson> },
        #[serde(rename = "and")]
        And { filters: &'a Vec<FilterJson> },
        #[serde(rename = "not")]
        Not { filter: &'a Box<FilterJson> },
    }

    impl Serialize for FilterJson {
        fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
            S: Serializer {
            match self {
                FilterJson::Equal(v) => FilterJsonForSerialize::Equal(v).serialize(serializer),
                FilterJson::Range(v) => FilterJsonForSerialize::Range(v).serialize(serializer),
                FilterJson::Or(v) => FilterJsonForSerialize::Or { filters: v }.serialize(serializer),
                FilterJson::And(v) => FilterJsonForSerialize::And { filters: v }.serialize(serializer),
                FilterJson::Not(v) => FilterJsonForSerialize::Not { filter: v }.serialize(serializer),
            }
        }
    }

    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum FilterJsonForDeserialize {
        #[serde(rename = "equal")]
        Equal(EqualFilter),
        #[serde(rename = "range")]
        Range(RangeFilter),
        #[serde(rename = "or")]
        Or { filters: Vec<FilterJson> },
        #[serde(rename = "and")]
        And { filters: Vec<FilterJson> },
        #[serde(rename = "not")]
        Not { filter: Box<FilterJson> },
    }

    impl<'de> Deserialize<'de> for FilterJson {
        fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
            D: Deserializer<'de> {
            Ok(
                match <FilterJsonForDeserialize as Deserialize>::deserialize(deserializer)? {
                    FilterJsonForDeserialize::Equal(v) => FilterJson::Equal(v),
                    FilterJsonForDeserialize::Range(v) => FilterJson::Range(v),
                    FilterJsonForDeserialize::Or { filters: v } => FilterJson::Or(v),
                    FilterJsonForDeserialize::And { filters: v } => FilterJson::And(v),
                    FilterJsonForDeserialize::Not { filter: v } => FilterJson::Not(v),
                }
            )
        }
    }

    pub fn bool_not(f: &bool) -> bool {
        !*f
    }

    pub fn bool_false() -> bool {
        false
    }
}
