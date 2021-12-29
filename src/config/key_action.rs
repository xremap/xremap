use crate::config::key::Key;
use serde::Deserialize;

// Values in `modmap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum KeyAction {
    Key(Key),
    MultiPurposeKey(MultiPurposeKey),
}

#[derive(Clone, Debug, Deserialize)]
pub struct MultiPurposeKey {
    pub held: Key,
    pub alone: Key,
    #[serde(default = "MultiPurposeKey::default_alone_timeout_millis")]
    pub alone_timeout_millis: u64,
}

impl MultiPurposeKey {
    fn default_alone_timeout_millis() -> u64 {
        1000
    }
}
