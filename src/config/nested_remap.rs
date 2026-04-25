use super::keymap_action::Actions;
use crate::config::application::deserialize_string_or_vec;
use crate::config::key::parse_key;
use crate::config::key_press::KeyPress;
use crate::config::keymap_action::KeymapAction;
use evdev::KeyCode as Key;
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Remap {
    pub remap: HashMap<KeyPress, Vec<KeymapAction>>,
    pub timeout: Option<Duration>,
    pub timeout_key: Option<Vec<Key>>,
}

// Used only for deserialization
#[derive(Debug, Deserialize)]
pub struct RemapActions {
    pub remap: HashMap<KeyPress, Actions>,
    pub timeout_millis: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub timeout_key: Option<Vec<String>>,
}

pub fn deserialize_nested_remap<'de, D>(deserializer: D) -> Result<Remap, D::Error>
where
    D: Deserializer<'de>,
{
    let action = RemapActions::deserialize(deserializer)?;
    Ok(Remap {
        remap: action.remap.into_iter().map(|(k, v)| (k, v.into_vec())).collect(),
        timeout: action.timeout_millis.map(Duration::from_millis),
        timeout_key: if let Some(keys) = action.timeout_key {
            let parsed_keys: Result<Vec<_>, _> = keys.into_iter().map(|key| parse_key(&key)).collect();
            match parsed_keys {
                Ok(parsed_keys) => Some(parsed_keys),
                Err(e) => return Err(de::Error::custom(e.to_string())),
            }
        } else {
            None
        },
    })
}
