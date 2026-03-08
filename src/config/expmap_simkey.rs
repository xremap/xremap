use crate::config::deserialize_keys;
use crate::config::expmap_operator::{deserialize_expmap_actions, ExpmapAction};
use evdev::KeyCode as Key;
use serde::Deserialize;
use serde_with::{serde_as, DurationMilliSeconds};
use std::time::Duration;

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct Simkey {
    #[serde(deserialize_with = "deserialize_keys")]
    pub keys: Vec<Key>,
    #[serde(deserialize_with = "deserialize_expmap_actions")]
    pub actions: Vec<ExpmapAction>,
    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(default = "default_symkey_timeout")]
    pub timeout: Duration,
}

fn default_symkey_timeout() -> Duration {
    Duration::from_millis(30)
}
