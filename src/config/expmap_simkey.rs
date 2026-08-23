use crate::config::deserialize_keys;
use crate::config::deserializers::deserialize_duration;
use crate::config::expmap_operator::{deserialize_expmap_actions, ExpmapAction};
use evdev::KeyCode as Key;
use serde::Deserialize;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
pub struct Simkey {
    #[serde(deserialize_with = "deserialize_keys")]
    pub keys: Vec<Key>,
    #[serde(deserialize_with = "deserialize_expmap_actions")]
    pub actions: Vec<ExpmapAction>,
    #[serde(default = "default_symkey_timeout", deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}

fn default_symkey_timeout() -> Duration {
    Duration::from_millis(30)
}
