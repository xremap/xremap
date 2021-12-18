use crate::config::action::Action;
use crate::config::keypress::KeyPress;
use crate::config::wm_class::WMClass;
use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    pub name: String,
    #[serde(deserialize_with = "keymap_remap")]
    pub remap: HashMap<KeyPress, Vec<Action>>,
    pub wm_class: Option<WMClass>,
}

fn keymap_remap<'de, D>(deserializer: D) -> Result<HashMap<KeyPress, Vec<Action>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct KeymapRemap;

    impl<'de> Visitor<'de> for KeymapRemap {
        type Value = HashMap<KeyPress, Vec<Action>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map from string to strings or maps")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut keymap = HashMap::new();

            while let Some(keypress) = map.next_key::<KeyPress>()? {
                let actions = if let Ok(to) = map.next_value::<Action>() {
                    vec![to]
                } else {
                    let error: Box<dyn std::error::Error> = "map values must be strings or maps".into();
                    return Err(error).map_err(M::Error::custom);
                };
                keymap.insert(keypress, actions);
            }

            Ok(keymap)
        }
    }

    deserializer.deserialize_any(KeymapRemap)
}
