use crate::config::action::Action;
use crate::config::action::Actions;
use crate::config::application::Application;
use crate::config::key_press::KeyPress;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<KeyPress, Vec<Action>>,
    pub application: Option<Application>,
}

fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<KeyPress, Vec<Action>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct KeymapRemap;

    impl<'de> Visitor<'de> for KeymapRemap {
        type Value = HashMap<KeyPress, Vec<Action>>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("map from string to strings or maps")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut keymap = HashMap::new();

            while let Some(key_press) = map.next_key::<KeyPress>()? {
                let actions = match map.next_value::<Actions>()? {
                    Actions::Action(action) => vec![action],
                    Actions::Actions(actions) => actions,
                };
                keymap.insert(key_press, actions);
            }

            Ok(keymap)
        }
    }

    deserializer.deserialize_any(KeymapRemap)
}
