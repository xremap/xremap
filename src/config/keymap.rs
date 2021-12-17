use crate::config::keypress::{parse_keypress, KeyPress};
use crate::config::wm_class::WMClass;
use serde::de::{value, Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    pub name: String,
    #[serde(deserialize_with = "keymap_remap")]
    pub remap: HashMap<KeyPress, Vec<KeyPress>>,
    pub wm_class: Option<WMClass>,
}

// TODO: Add Action trait

fn keymap_remap<'de, D>(deserializer: D) -> Result<HashMap<KeyPress, Vec<KeyPress>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct KeymapRemap;

    impl<'de> Visitor<'de> for KeymapRemap {
        type Value = HashMap<KeyPress, Vec<KeyPress>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map from string to string, array, or map")
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let remap: HashMap<String, String> =
                Deserialize::deserialize(value::MapAccessDeserializer::new(map))?;
            let mut keymap = HashMap::new();

            for (from, to) in remap.iter() {
                let from_keymap = parse_keypress(&from).map_err(M::Error::custom)?;
                let to_keymap = parse_keypress(&to).map_err(M::Error::custom)?;
                keymap.insert(from_keymap, vec![to_keymap]);
            }

            Ok(keymap)
        }
    }

    deserializer.deserialize_any(KeymapRemap)
}
