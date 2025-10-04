use crate::config::key::deserialize_key;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DurationMilliSeconds};
use std::time::Duration;

use super::{
    deserialize_virtual_modifiers,
    keymap_action::{Actions, KeymapAction},
};

// Values in `modmap.remap`
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ModmapAction {
    Keys(Keys),
    MultiPurposeKey(MultiPurposeKey),
    PressReleaseKey(PressReleaseKey),
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct MultiPurposeKey {
    #[serde(alias = "held")]
    pub hold: Keys,
    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(
        default = "default_hold_threshold",
        alias = "hold_threshold_millis",
        alias = "held_threshold_millis"
    )]
    pub hold_threshold: Duration,
    #[serde(alias = "alone")]
    pub tap: Keys,
    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(
        default = "default_tap_timeout",
        alias = "tap_timeout_millis",
        alias = "alone_timeout_millis"
    )]
    pub tap_timeout: Duration,
    #[serde(default = "default_free_hold")]
    pub free_hold: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PressReleaseKey {
    #[serde(default)]
    pub skip_key_event: bool,
    #[serde(default, deserialize_with = "deserialize_actions")]
    pub press: Vec<KeymapAction>,
    #[serde(default, deserialize_with = "deserialize_actions")]
    pub repeat: Vec<KeymapAction>,
    #[serde(default, deserialize_with = "deserialize_actions")]
    pub release: Vec<KeymapAction>,
}
// Used only for deserializing Vec<Keys>
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Keys {
    #[serde(deserialize_with = "deserialize_key")]
    Key(Key),
    #[serde(deserialize_with = "deserialize_virtual_modifiers")]
    Keys(Vec<Key>),
}

impl Keys {
    pub fn into_vec(self) -> Vec<Key> {
        match self {
            Keys::Key(key) => vec![key],
            Keys::Keys(keys) => keys,
        }
    }
}

pub fn deserialize_actions<'de, D>(deserializer: D) -> Result<Vec<KeymapAction>, D::Error>
where
    D: Deserializer<'de>,
{
    let actions = Actions::deserialize(deserializer)?;
    Ok(actions.into_vec())
}

fn default_tap_timeout() -> Duration {
    Duration::from_millis(1000)
}

fn default_hold_threshold() -> Duration {
    Duration::from_millis(0)
}

fn default_free_hold() -> bool {
    false // NOTE: should default to true ?!
          // up to maintainer
}
