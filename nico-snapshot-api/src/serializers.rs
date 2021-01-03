use serde::{Serializer, Deserializer, Serialize, Deserialize};
use serde::de::Visitor;
use std::marker::PhantomData;
use serde::export::fmt::Display;
use itertools::Itertools;
use std::str::FromStr;
use std::time::Duration;
use reqwest::StatusCode;
use crate::FieldName;

macro_rules! de_or_serialize_module {
    ( $( $acc : vis mod $name: ident for $type: ty = $expr: expr )* ) => {
    $(
        $acc mod $name {
            use super::*;
            use serde::{Serializer, Deserializer};

            #[allow(dead_code)]
            pub fn serialize<S>(value: &$type, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                let serialize_impl = $expr;
                serialize_impl.serialize(value, serializer)
            }

            #[allow(dead_code)]
            pub fn deserialize<'de, D>(deserializer: D) -> Result<$type, D::Error> where
                D: Deserializer<'de> {
                let deserialize_impl = $expr;
                deserialize_impl.deserialize(deserializer)
            }
        }
    )*
    };
}

macro_rules! de_or_serialize_struct {
    (
    $(
    $acc : vis struct $name: ident for $type: ty {
        fn serialize($value_serialize: ident, $serializer: ident) {
            $( $serialize:stmt )*
        }
        fn deserialize($deserializer: ident) {
            $( $deserialize:stmt )*
        }
    }
    )*
    ) => {
    $(
        $acc struct $name;
        impl $name {
            fn new() -> $name {
                $name {}
            }
        }

        impl SerializerImpl for $name {
           type Input = $type;
           fn serialize<S>(&self, $value_serialize: &Self::Input, $serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
               $( $serialize )*
           }
        }

        impl DeserializerImpl for $name {
           type Output = $type;
           fn deserialize<'de, D>(&self, $deserializer: D) -> Result<Self::Output, D::Error> where D: Deserializer<'de> {
               $( $deserialize )*
           }
        }
    )*
    };
}

de_or_serialize_module! {
    pub(crate) mod comma_string_vec for Vec<String> = SeparatedStrings::new(",")
    pub(crate) mod comma_field_name_vec for Vec<FieldName> = SeparatedStrings::new(",")
    pub(crate) mod space_string_vec_opt for Option<Vec<String>> = ForOption::new(SeparatedStrings::new(" "))
    pub(crate) mod duration_opt_seconds for Option<Duration> = ForOption::new(DurtionSeconds::new())
    pub(crate) mod status_code_serde for StatusCode = StatusCodeSerde::new()
}

de_or_serialize_struct! {

struct DurtionSeconds for Duration {
    fn serialize(value, serializer) {
        serializer.serialize_u64(value.as_secs())
    }

    fn deserialize(deserializer) {
        Ok(Duration::from_secs(<u64 as Deserialize>::deserialize(deserializer)?))
    }
}

struct StatusCodeSerde for StatusCode {
    fn serialize(code, serializer) {
        serializer.serialize_u16(code.as_u16())
    }

    fn deserialize(deserializer) {{
        use serde::de::{Error, Unexpected};

        let code = <u16 as Deserialize>::deserialize(deserializer)?;
        if let Ok(code) = StatusCode::from_u16(code) {
            Ok(code)
        } else {
            Err(D::Error::invalid_value(Unexpected::Unsigned(code as u64), &"100..=999"))
        }
    }}
}

}

trait SerializerImpl {
    type Input;
    fn serialize<S>(&self, value: &Self::Input, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer;
}

trait DeserializerImpl {
    type Output;
    fn deserialize<'de, D>(&self, deserializer: D) -> Result<Self::Output, D::Error> where D: Deserializer<'de>;
}

pub struct ForOption<Base> {
    base: Base
}

impl <Base> ForOption<Base> {
    fn new(base: Base) -> ForOption<Base> {
        ForOption { base }
    }
}

impl <V, Base: SerializerImpl<Input = V>> SerializerImpl for ForOption<Base> {
    type Input = Option<V>;
    fn serialize<S>(&self, value: &Option<V>, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where S: Serializer {
        match value {
            None => serializer.serialize_none(),
            Some(value) => {
                struct Wrapper<'a, V, Base> {
                    value: &'a V,
                    base: &'a Base,
                }
                impl <'a, V, Base : SerializerImpl<Input = V>> Serialize for Wrapper<'a, V, Base> {
                    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
                        S: Serializer {
                        self.base.serialize(self.value, serializer)
                    }
                }
                serializer.serialize_some(&Wrapper { value, base: &self.base })
            }
        }
    }
}

impl <V, Base: DeserializerImpl<Output = V>> DeserializerImpl for ForOption<Base> {
    type Output = Option<V>;
    fn deserialize<'de, D>(&self, deserializer: D) -> Result<Option<V>, <D as Deserializer<'de>>::Error> where D: Deserializer<'de> {
        use serde::de::Error;
        struct Wrapper<'a, Base> {
            base: &'a Base
        }
        impl<'de, 'a, V, Base: DeserializerImpl<Output = V>> Visitor<'de> for Wrapper<'a, Base> {
            type Value = Option<V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("option")
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
            {
                self.base.deserialize(deserializer).map(|value| Some(value))
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
                where
                    E: Error,
            {
                Ok(None)
            }
        }

        deserializer.deserialize_option(Wrapper { base: &self.base })
    }
}

pub struct SeparatedStrings<'a, Value> {
    splitter: &'a str,
    marker: PhantomData<Value>,
}

impl <'a, Value> SeparatedStrings<'a, Value> {
    fn new(splitter: &'a str) -> SeparatedStrings<'a, Value> {
        SeparatedStrings {
            splitter,
            marker: PhantomData::default()
        }
    }
}

impl <'a, Value: Display> SerializerImpl for SeparatedStrings<'a, Value> {
    type Input = Vec<Value>;

    fn serialize<S>(&self, value: &Self::Input, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let str = value.iter().join(self.splitter);
        serializer.serialize_str(&str)
    }
}

impl <'a, Value> DeserializerImpl for SeparatedStrings<'a, Value>
    where Value: FromStr, Value::Err: Display {
    type Output = Vec<Value>;

    fn deserialize<'de, D>(&self, deserializer: D) -> Result<Self::Output, D::Error> where D: Deserializer<'de> {
        use serde::de::Error;

        let str = <String as Deserialize>::deserialize(deserializer)?;
        let mut vec = Vec::<Value>::new();
        for x in str.split(self.splitter) {
            match Value::from_str(x) {
                Ok(v) => {
                    vec.push(v)
                }
                Err(err) => {
                    return Err(D::Error::custom(err))
                }
            }
        }
        Ok(vec)
    }
}
