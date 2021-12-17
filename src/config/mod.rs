mod key;

extern crate serde_yaml;

use evdev::Key;
use serde::de::{value, Error, MapAccess, SeqAccess, Visitor};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::{error, fmt, fs};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub modmap: Vec<Modmap>,
    pub keymap: Vec<Keymap>,
}

#[derive(Debug, Deserialize)]
pub struct Modmap {
    pub name: String,
    #[serde(deserialize_with = "modmap_remap")]
    pub remap: HashMap<Key, Key>,
    pub wm_class: Option<WMClass>,
}

#[derive(Debug, Deserialize)]
pub struct Keymap {
    pub name: String,
    pub remap: HashMap<String, String>,
    pub wm_class: Option<WMClass>,
}

// TODO: Use trait to allow only either `only` or `not`
#[derive(Debug, Deserialize)]
pub struct WMClass {
    #[serde(default, deserialize_with = "string_or_vec")]
    pub only: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_vec")]
    pub not: Option<Vec<String>>,
}

pub fn load_config(filename: &str) -> Result<Config, Box<dyn error::Error>> {
    let yaml = fs::read_to_string(&filename)?;
    let config: Config = serde_yaml::from_str(&yaml)?;
    return Ok(config);
}

fn modmap_remap<'de, D>(deserializer: D) -> Result<HashMap<Key, Key>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ModmapRemap;

    impl<'de> Visitor<'de> for ModmapRemap {
        type Value = HashMap<Key, Key>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map of string to string")
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let remap: HashMap<String, String> =
                Deserialize::deserialize(value::MapAccessDeserializer::new(map))?;
            let mut key_remap = HashMap::new();
            for (from, to) in remap.iter() {
                let from_key = key::parse_key(&from).map_err(M::Error::custom)?;
                let to_key = key::parse_key(&to).map_err(M::Error::custom)?;
                key_remap.insert(from_key, to_key);
            }
            Ok(key_remap)
        }
    }

    deserializer.deserialize_any(ModmapRemap)
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
