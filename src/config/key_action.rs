use crate::config::key::deserialize_key;
use evdev::Key;
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};
use std::time::Duration;

// Values in `modmap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum KeyAction {
    #[serde(deserialize_with = "deserialize_key")]
    Key(Key),
    MultiPurposeKey(MultiPurposeKey),
    CommandKey(CommandKey),
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct MultiPurposeKey {
    #[serde(deserialize_with = "deserialize_key")]
    pub held: Key,
    #[serde(deserialize_with = "deserialize_key")]
    pub alone: Key,
    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(default = "default_alone_timeout", rename = "alone_timeout_millis")]
    pub alone_timeout: Duration,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CommandKey {
    pub press: Vec<String>,
    pub release: Vec<String>,
}

fn default_alone_timeout() -> Duration {
    Duration::from_millis(1000)
}
