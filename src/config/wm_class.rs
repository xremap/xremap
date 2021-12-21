use serde::de::{value, SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::fmt;

// TODO: Use trait to allow only either `only` or `not`
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WMClass {
    #[serde(default, deserialize_with = "string_or_vec")]
    pub only: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec")]
    pub not: Option<Vec<String>>,
}

fn string_or_vec<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(vec![s.to_owned()]))
        }

        fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let result: Vec<String> = Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))?;
            Ok(Some(result))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}
