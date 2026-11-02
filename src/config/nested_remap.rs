use crate::config::deserializers::{VecOrSingle, VectorOrSingleOrNull};
use crate::config::key_combo::KeyCombo;
use crate::config::keymap_action::{KeymapAction, ParseKeymapAction};
use crate::config::modmap::KeyWrapper;
use evdev::KeyCode as Key;
use indexmap::IndexMap;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Remap {
    pub remap: IndexMap<KeyCombo, Vec<KeymapAction>>,
    pub timeout: Option<Duration>,
    // Option is retained here, because it shows the user wants the default. The value can
    // only be determined at remap-time, because it depends on mod_trigger and any-key behavior.
    pub timeout_key: Option<Vec<Key>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParseRemap {
    pub remap: IndexMap<KeyCombo, VectorOrSingleOrNull<ParseKeymapAction>>,
    pub timeout_millis: Option<u64>,
    #[serde(default, deserialize_with = "deserialize_timeout_key")]
    pub timeout_key: Option<Vec<KeyWrapper>>,
}

pub fn deserialize_timeout_key<'de, D>(deserializer: D) -> Result<Option<Vec<KeyWrapper>>, D::Error>
where
    D: Deserializer<'de>,
{
    // This disallows null, even though Option<T> would regard
    // missing fields and null the same.
    Ok(Some(VecOrSingle::<KeyWrapper>::deserialize(deserializer)?.into_vec()))
}

impl ParseRemap {
    pub fn finish(self) -> Remap {
        Remap {
            remap: self
                .remap
                .into_iter()
                .map(|(combo, actions)| (combo, ParseKeymapAction::finish(actions.into_vec())))
                .collect(),
            timeout: self.timeout_millis.map(Duration::from_millis),
            timeout_key: self
                .timeout_key
                .map(|keys| keys.into_iter().map(|KeyWrapper(key)| key).collect()),
        }
    }
}
