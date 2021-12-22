use crate::config::key::parse_key;
use crate::config::application::Application;
use evdev::Key;
use serde::de::{value, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Modmap {
    pub name: String,
    #[serde(deserialize_with = "modmap_remap")]
    pub remap: HashMap<Key, Key>,
    pub application: Option<Application>,
}

fn modmap_remap<'de, D>(deserializer: D) -> Result<HashMap<Key, Key>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ModmapRemap;

    impl<'de> Visitor<'de> for ModmapRemap {
        type Value = HashMap<Key, Key>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map from string to string")
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let remap: HashMap<String, String> = Deserialize::deserialize(value::MapAccessDeserializer::new(map))?;
            let mut modmap = HashMap::new();

            for (from, to) in remap.iter() {
                let from_key = parse_key(&from).map_err(M::Error::custom)?;
                let to_key = parse_key(&to).map_err(M::Error::custom)?;
                modmap.insert(from_key, to_key);
            }

            Ok(modmap)
        }
    }

    deserializer.deserialize_any(ModmapRemap)
}
