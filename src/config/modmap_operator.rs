use super::deserialize_keys;
use super::keymap_action::KeymapAction;
use crate::config::deserializers::{deserialize_duration, deserialize_key, VectorOrSingleOrNull};
use crate::config::key::parse_key;
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent};
use crate::event_handler::{modifiers_first, modifiers_last};
use crate::KeyValue;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::rc::Rc;
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum ModmapOperator {
    Keys(KeyAction),
    MultiPurposeKey(MultiPurposeKey),
    PressReleaseKey(PressReleaseKey),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ParseModmapOperator {
    Keys(Keys),
    MultiPurposeKey(ParseMultiPurposeKey),
    PressReleaseKey(ParsePressReleaseKey),
}

impl ParseModmapOperator {
    pub fn finish(self, bypass: bool) -> ModmapOperator {
        match self {
            ParseModmapOperator::Keys(definition) => ModmapOperator::Keys(definition.finish(bypass)),
            ParseModmapOperator::MultiPurposeKey(definition) => {
                ModmapOperator::MultiPurposeKey(definition.finish(bypass))
            }
            ParseModmapOperator::PressReleaseKey(definition) => {
                ModmapOperator::PressReleaseKey(definition.finish(bypass))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MultiPurposeKey {
    pub hold: KeyAction,
    pub tap: KeyAction,
    pub hold_threshold: Duration,
    pub tap_timeout: Duration,
    pub free_hold: bool,
    pub interruptable: Interruptable,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParseMultiPurposeKey {
    #[serde(alias = "held")]
    pub hold: Keys,
    #[serde(alias = "alone")]
    pub tap: Keys,
    #[serde(
        default = "default_hold_threshold",
        alias = "hold_threshold_millis",
        alias = "held_threshold_millis",
        deserialize_with = "deserialize_duration"
    )]
    pub hold_threshold: Duration,
    #[serde(
        default = "default_tap_timeout",
        alias = "tap_timeout_millis",
        alias = "alone_timeout_millis",
        deserialize_with = "deserialize_duration"
    )]
    pub tap_timeout: Duration,
    #[serde(default)]
    pub free_hold: bool,
    #[serde(default)]
    pub interruptable: Interruptable,
}

impl ParseMultiPurposeKey {
    pub fn finish(self, bypass: bool) -> MultiPurposeKey {
        MultiPurposeKey {
            tap: self.tap.finish(bypass),
            hold: self.hold.finish(bypass),
            hold_threshold: self.hold_threshold,
            tap_timeout: self.tap_timeout,
            free_hold: self.free_hold,
            interruptable: self.interruptable,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PressReleaseKey {
    pub skip_key_event: bool,
    pub press: Vec<KeymapAction>,
    pub repeat: Vec<KeymapAction>,
    pub release: Vec<KeymapAction>,
    pub bypass: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParsePressReleaseKey {
    #[serde(default)]
    pub skip_key_event: bool,
    #[serde(default, deserialize_with = "deserialize_actions")]
    pub press: Vec<KeymapAction>,
    #[serde(default, deserialize_with = "deserialize_actions")]
    pub repeat: Vec<KeymapAction>,
    #[serde(default, deserialize_with = "deserialize_actions")]
    pub release: Vec<KeymapAction>,
}

impl ParsePressReleaseKey {
    pub fn finish(self, bypass: bool) -> PressReleaseKey {
        PressReleaseKey {
            skip_key_event: self.skip_key_event,
            press: self.press,
            repeat: self.repeat,
            release: self.release,
            bypass,
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyAction {
    keys: Vec<Key>,
    bypass: bool,
}

impl KeyAction {
    /// For emitting key-to-key remaps.
    pub fn emit(&self, device: Rc<InputDeviceInfo>, value: KeyValue) -> Vec<Event> {
        self.keys
            .iter()
            .map(|key| {
                if self.bypass {
                    Event::ByPass(KeyEvent::new(*key, value))
                } else {
                    Event::KeyEvent(device.clone(), KeyEvent::new(*key, value))
                }
            })
            .collect()
    }

    /// For emitting MultiPurposeKey press or repeat.
    pub fn emit_mul_pur(&self, device: &Rc<InputDeviceInfo>, value: KeyValue) -> Vec<Event> {
        let mut keys = self.keys.clone();
        keys.sort_by(modifiers_first);
        keys.into_iter()
            .map(|key| {
                if self.bypass {
                    Event::ByPass(KeyEvent::new(key, value))
                } else {
                    Event::KeyEvent(device.clone(), KeyEvent::new(key, value))
                }
            })
            .collect()
    }

    pub fn emit_mul_pur_release(&self, device: &Rc<InputDeviceInfo>) -> Vec<Event> {
        let mut keys = self.keys.clone();
        keys.sort_by(modifiers_last);
        keys.into_iter()
            .map(|key| {
                if self.bypass {
                    Event::ByPass(KeyEvent::new(key, KeyValue::Release))
                } else {
                    Event::KeyEvent(device.clone(), KeyEvent::new(key, KeyValue::Release))
                }
            })
            .collect()
    }
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
    pub fn finish(self, bypass: bool) -> KeyAction {
        KeyAction {
            keys: self.into_vec(),
            bypass,
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
        Interruptable::Not {
            // Multipurpose key is not interrupted by mouse movement by default.
            not: Keys::Keys(vec![
                parse_key("XRIGHTCURSOR").unwrap(),
                parse_key("XLEFTCURSOR").unwrap(),
                parse_key("XDOWNCURSOR").unwrap(),
                parse_key("XUPCURSOR").unwrap(),
            ]),
        }
    }
}

pub fn deserialize_actions<'de, D>(deserializer: D) -> Result<Vec<KeymapAction>, D::Error>
where
    D: Deserializer<'de>,
{
    let actions = VectorOrSingleOrNull::<KeymapAction>::deserialize(deserializer)?;
    Ok(actions.into_vec())
}

fn default_tap_timeout() -> Duration {
    Duration::from_millis(1000)
}

fn default_hold_threshold() -> Duration {
    Duration::from_millis(0)
}
