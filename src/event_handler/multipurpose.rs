use evdev::KeyCode as Key;
use std::cmp::Ordering;
use std::time::Instant;

use crate::config::modmap_action::Keys;
use crate::event::KeyValue;

use super::MODIFIER_KEYS;

#[derive(Debug)]
pub struct MultiPurposeKeyState {
    pub held: Keys,
    pub alone: Keys,

    pub alone_timeout_at: Option<Instant>,
    pub held_down: bool,
}

impl MultiPurposeKeyState {
    pub fn repeat(&mut self) -> Vec<(Key, KeyValue)> {
        match self.alone_timeout_at {
            Some(alone_timeout_at) if Instant::now() < alone_timeout_at => {
                vec![]
            }
            Some(_) => {
                self.alone_timeout_at = None;
                self.held_down = true;
                let mut keys = self.held.clone().into_vec();
                keys.sort_by(modifiers_first);
                keys.into_iter().map(|key| (key, KeyValue::Press)).collect()
            }
            None => {
                let mut keys = self.held.clone().into_vec();
                keys.sort_by(modifiers_first);
                keys.into_iter().map(|key| (key, KeyValue::Repeat)).collect()
            }
        }
    }

    pub fn release(&self) -> Vec<(Key, KeyValue)> {
        match self.alone_timeout_at {
            Some(alone_timeout_at) if Instant::now() < alone_timeout_at => self.press_and_release(&self.alone),
            Some(_) => self.press_and_release(&self.held),
            None => match self.held_down {
                true => {
                    let mut release_keys = self.held.clone().into_vec();
                    release_keys.sort_by(modifiers_last);
                    release_keys.into_iter().map(|key| (key, KeyValue::Release)).collect()
                }
                false => self.press_and_release(&self.alone),
            },
        }
    }

    pub fn force_held(&mut self) -> Vec<(Key, KeyValue)> {
        let press = match self.alone_timeout_at {
            Some(_) => {
                self.alone_timeout_at = None;
                self.held_down = true;
                true
            }
            None => {
                if !self.held_down {
                    self.held_down = true;
                    true
                } else {
                    false
                }
            }
        };

        if press {
            let mut keys = self.held.clone().into_vec();
            keys.sort_by(modifiers_first);
            keys.into_iter().map(|key| (key, KeyValue::Press)).collect()
        } else {
            vec![]
        }
    }

    fn press_and_release(&self, keys_to_use: &Keys) -> Vec<(Key, KeyValue)> {
        let mut release_keys = keys_to_use.clone().into_vec();
        release_keys.sort_by(modifiers_last);
        let release_events: Vec<(Key, KeyValue)> =
            release_keys.into_iter().map(|key| (key, KeyValue::Release)).collect();

        let mut press_keys = keys_to_use.clone().into_vec();
        press_keys.sort_by(modifiers_first);
        let mut events: Vec<(Key, KeyValue)> = press_keys.into_iter().map(|key| (key, KeyValue::Press)).collect();
        events.extend(release_events);
        events
    }
}

pub fn modifiers_first(a: &Key, b: &Key) -> Ordering {
    if MODIFIER_KEYS.contains(a) {
        if MODIFIER_KEYS.contains(b) {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    } else if MODIFIER_KEYS.contains(b) {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

pub fn modifiers_last(a: &Key, b: &Key) -> Ordering {
    modifiers_first(a, b).reverse()
}
