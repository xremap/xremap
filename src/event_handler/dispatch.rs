use std::error::Error;
use std::time::Duration;

use evdev::KeyCode as Key;
use log::debug;
use nix::sys::time::TimeSpec;
use nix::sys::timerfd::{Expiration, TimerSetTimeFlags};

use crate::action::Action;
use crate::config::key_press::{KeyPress, Modifier};
use crate::config::keymap::build_override_table;
use crate::config::keymap_action::KeymapAction;
use crate::config::remap::Remap;
use crate::event::{KeyEvent, KeyValue};

use super::{EventHandler, TaggedAction};

impl EventHandler {
    pub fn send_keys(&mut self, keys: &Vec<Key>, value: KeyValue) -> Vec<Action> {
        let mut out = Vec::with_capacity(keys.len());
        for key in keys {
            out.push(self.send_key(key, value));
        }
        out
    }

    pub fn send_key(&mut self, key: &Key, value: KeyValue) -> Action {
        let event = KeyEvent::new(*key, value);
        Action::KeyEvent(event)
    }

    pub fn send_action(&mut self, action: Action) -> Action {
        action
    }

    pub fn run_command(&mut self, command: Vec<String>) -> Action {
        Action::Command(command)
    }

    pub fn dispatch_actions(&mut self, actions: &Vec<TaggedAction>, key: &Key) -> Result<Vec<Action>, Box<dyn Error>> {
        let mut out = Vec::with_capacity(actions.len());
        for action in actions {
            out.extend(self.dispatch_action(action, key)?);
        }
        Ok(out)
    }

    pub fn dispatch_action(&mut self, action: &TaggedAction, key: &Key) -> Result<Vec<Action>, Box<dyn Error>> {
        let actions = match &action.action {
            KeymapAction::KeyPressAndRelease(key_press) => self.send_key_press_and_release(key_press),
            KeymapAction::KeyPress(key) => vec![self.send_key(key, KeyValue::Press)],
            KeymapAction::KeyRepeat(key) => vec![self.send_key(key, KeyValue::Repeat)],
            KeymapAction::KeyRelease(key) => vec![self.send_key(key, KeyValue::Release)],
            KeymapAction::Remap(Remap {
                remap,
                timeout,
                timeout_key,
            }) => {
                let set_timeout = self.override_remaps.is_empty();
                self.override_remaps
                    .push(build_override_table(remap, action.exact_match));

                if set_timeout {
                    if let Some(timeout) = timeout {
                        let expiration = Expiration::OneShot(TimeSpec::from_duration(*timeout));

                        self.override_timer.unset()?;
                        self.override_timer.set(expiration, TimerSetTimeFlags::empty())?;
                        self.override_timeout_key = timeout_key.clone().or_else(|| Some(vec![*key]))
                    }
                }
                vec![]
            }
            KeymapAction::Launch(command) => vec![self.run_command(command.clone())],
            KeymapAction::SetMode(mode) => {
                self.mode = mode.clone();
                debug!("mode: {}", mode);
                vec![]
            }
            KeymapAction::SetMark(set) => {
                self.mark_set = *set;
                vec![]
            }
            KeymapAction::WithMark(key_press) => self.send_key_press_and_release(&self.with_mark(key_press)),
            KeymapAction::EscapeNextKey(escape_next_key) => {
                self.escape_next_key = *escape_next_key;
                vec![]
            }
            KeymapAction::Sleep(millis) => vec![self.send_action(Action::Delay(Duration::from_millis(*millis)))],
            KeymapAction::SetExtraModifiers(keys) => {
                self.extra_modifiers.clear();
                for key in keys {
                    self.extra_modifiers.insert(*key);
                }
                vec![]
            }
        };
        Ok(actions)
    }

    pub fn send_key_press_and_release(&mut self, key_press: &KeyPress) -> Vec<Action> {
        let (mut extra_modifiers, mut missing_modifiers) = self.diff_modifiers(&key_press.modifiers);
        extra_modifiers.retain(|key| super::MODIFIER_KEYS.contains(key) && !self.extra_modifiers.contains(key));
        missing_modifiers.retain(|key| super::MODIFIER_KEYS.contains(key));

        let mut out = Vec::new();
        out.extend(self.send_keys(&missing_modifiers, KeyValue::Press));
        out.extend(self.send_keys(&extra_modifiers, KeyValue::Release));

        out.push(self.send_key(&key_press.key, KeyValue::Press));
        out.push(self.send_key(&key_press.key, KeyValue::Release));

        out.push(self.send_action(Action::Delay(self.keypress_delay)));

        out.extend(self.send_keys(&extra_modifiers, KeyValue::Press));
        out.push(self.send_action(Action::Delay(self.keypress_delay)));
        out.extend(self.send_keys(&missing_modifiers, KeyValue::Release));

        out
    }

    pub fn with_mark(&self, key_press: &KeyPress) -> KeyPress {
        if self.mark_set && !self.match_modifier(&Modifier::Shift) {
            let mut modifiers = key_press.modifiers.clone();
            modifiers.push(Modifier::Shift);
            KeyPress {
                key: key_press.key,
                modifiers,
            }
        } else {
            key_press.clone()
        }
    }
}
