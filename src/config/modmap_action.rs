use super::deserialize_keys;
use super::keymap_action::{Actions, KeymapAction};
use crate::config::key::deserialize_key;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DurationMilliSeconds};
use std::time::Duration;

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
    #[serde(alias = "alone")]
    pub tap: Keys,
    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(
        default = "default_hold_threshold",
        alias = "hold_threshold_millis",
        alias = "held_threshold_millis"
    )]
    pub hold_threshold: Duration,
    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(
        default = "default_tap_timeout",
        alias = "tap_timeout_millis",
        alias = "alone_timeout_millis"
    )]
    pub tap_timeout: Duration,
    #[serde(default = "default_free_hold")]
    pub free_hold: bool,
    #[serde(default)]
    pub interruptable: Interruptable,
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
// Used only for deserializing
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Keys {
    #[serde(deserialize_with = "deserialize_key")]
    Key(Key),
    #[serde(deserialize_with = "deserialize_keys")]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Interruptable {
    All(bool),
    Only { only: Keys },
    Not { not: Keys },
}

impl Interruptable {
    pub fn is_interrupted_by(&self, key: Key) -> bool {
        match self {
            Interruptable::All(all) => *all,
            Interruptable::Only { only: Keys::Key(only) } => key == *only,
            Interruptable::Only { only: Keys::Keys(only) } => only.iter().any(|k| key == *k),
            Interruptable::Not { not: Keys::Key(not) } => key != *not,
            Interruptable::Not { not: Keys::Keys(not) } => not.iter().all(|k| key != *k),
        }
    }
}

impl Default for Interruptable {
    fn default() -> Self {
        Interruptable::All(true)
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
