extern crate serde_yaml;

use serde::de::{value, SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::error::Error;
use std::{fmt, fs};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub modmap: Vec<Modmap>,
    pub keymap: Vec<Keymap>,
}

#[derive(Debug, Deserialize)]
pub struct Modmap {
    pub name: String,
    pub remap: HashMap<String, String>,
    pub wm_class: Option<WMClass>,
}

#[derive(Debug, Deserialize)]
pub struct Keymap {
    pub name: String,
    pub remap: HashMap<String, String>,
    pub wm_class: Option<WMClass>,
}

#[derive(Debug, Deserialize)]
pub struct WMClass {
    // TODO: validate only either `only` or `not` is set
    #[serde(default, deserialize_with = "string_or_vec")]
    pub only: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec")]
    pub not: Option<Vec<String>>,
}

pub fn load_config(filename: &str) -> Result<Config, Box<dyn Error>> {
    let yaml = fs::read_to_string(&filename)?;
    let config: Config = serde_yaml::from_str(&yaml)?;
    return Ok(config);
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
            let result: Vec<String> =
                Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))?;
            Ok(Some(result))
        }
    }

    deserializer.deserialize_any(StringOrVec)
}
