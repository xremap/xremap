use evdev::KeyCode as Key;
use std::cmp::Ordering;
use std::time::Instant;

use crate::config::modmap_action::Keys;

use super::{MODIFIER_KEYS, PRESS, REPEAT, RELEASE};

#[derive(Debug)]
pub(super) struct MultiPurposeKeyState {
    pub(super) held: Keys,
    pub(super) alone: Keys,
    // Some if the first press is still delayed, None if already considered held.
    pub(super) alone_timeout_at: Option<Instant>,
    pub(super) held_down: bool,
}

impl MultiPurposeKeyState {
    pub(super) fn repeat(&mut self) -> Vec<(Key, i32)> {
        match self.alone_timeout_at {
            Some(alone_timeout_at) if Instant::now() < alone_timeout_at => {
                vec![] // still delay the press
            }
            Some(_) => {
                // timeout
                self.alone_timeout_at = None;
                self.held_down = true;
                let mut keys = self.held.clone().into_vec();
                keys.sort_by(modifiers_first);
                keys.into_iter().map(|key| (key, PRESS)).collect()
            }
            None => {
                let mut keys = self.held.clone().into_vec();
                keys.sort_by(modifiers_first);
                keys.into_iter().map(|key| (key, REPEAT)).collect()
            }
        }
    }

    pub(super) fn release(&self) -> Vec<(Key, i32)> {
        match self.alone_timeout_at {
            Some(alone_timeout_at) if Instant::now() < alone_timeout_at => self.press_and_release(&self.alone),
            Some(_) => self.press_and_release(&self.held),
            None => match self.held_down {
                true => {
                    let mut release_keys = self.held.clone().into_vec();
                    release_keys.sort_by(modifiers_last);
                    release_keys.into_iter().map(|key| (key, RELEASE)).collect()
                }
                false => self.press_and_release(&self.alone),
            },
        }
    }

    pub(super) fn force_held(&mut self) -> Vec<(Key, i32)> {
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
            keys.into_iter().map(|key| (key, PRESS)).collect()
        } else {
            vec![]
        }
    }

    fn press_and_release(&self, keys_to_use: &Keys) -> Vec<(Key, i32)> {
        let mut release_keys = keys_to_use.clone().into_vec();
        release_keys.sort_by(modifiers_last);
        let release_events: Vec<(Key, i32)> = release_keys.into_iter().map(|key| (key, RELEASE)).collect();

        let mut press_keys = keys_to_use.clone().into_vec();
        press_keys.sort_by(modifiers_first);
        let mut events: Vec<(Key, i32)> = press_keys.into_iter().map(|key| (key, PRESS)).collect();
        events.extend(release_events);
        events
    }
}

/// Orders modifier keys ahead of non-modifier keys.
/// Unfortunately the underlying type doesn't allow direct
/// comparison, but that's ok for our purposes.
pub(super) fn modifiers_first(a: &Key, b: &Key) -> Ordering {
    if MODIFIER_KEYS.contains(a) {
        if MODIFIER_KEYS.contains(b) {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    } else if MODIFIER_KEYS.contains(b) {
        Ordering::Greater
    } else {
        // Neither are modifiers
        Ordering::Equal
    }
}

pub(super) fn modifiers_last(a: &Key, b: &Key) -> Ordering {
    modifiers_first(a, b).reverse()
}
