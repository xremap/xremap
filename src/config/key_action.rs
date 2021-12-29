use crate::config::key::deserialize_key;
use evdev::Key;
use serde::Deserialize;

// Values in `modmap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum KeyAction {
    #[serde(deserialize_with = "deserialize_key")]
    Key(Key),
    MultiPurposeKey(MultiPurposeKey),
}

#[derive(Clone, Debug, Deserialize)]
pub struct MultiPurposeKey {
    #[serde(deserialize_with = "deserialize_key")]
    pub held: Key,
    #[serde(deserialize_with = "deserialize_key")]
    pub alone: Key,
    #[serde(default = "MultiPurposeKey::default_alone_timeout_millis")]
    pub alone_timeout_millis: u64,
}

impl MultiPurposeKey {
    fn default_alone_timeout_millis() -> u64 {
        1000
    }
}
