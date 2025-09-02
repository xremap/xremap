use std::error::Error;
use std::time::Duration;

use evdev::KeyCode as Key;
use nix::sys::time::TimeSpec;
use nix::sys::timerfd::{Expiration, TimerSetTimeFlags};

use crate::action::Action;
use crate::config::key_press::{KeyPress, Modifier};
use crate::config::keymap::build_override_table;
use crate::config::keymap_action::KeymapAction;
use crate::config::remap::Remap;
use crate::event::KeyEvent;

use super::{EventHandler, TaggedAction};

impl EventHandler {
    pub(super) fn send_keys(&mut self, keys: &Vec<Key>, value: i32) {
        for key in keys {
            self.send_key(key, value);
        }
    }

    pub(super) fn send_key(&mut self, key: &Key, value: i32) {
        // let event = InputEvent::new(EventType::KEY, key.code(), value);
        let event = KeyEvent::new_with(key.code(), value);
        self.send_action(Action::KeyEvent(event));
    }

    pub(super) fn send_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    pub(super) fn run_command(&mut self, command: Vec<String>) {
        self.send_action(Action::Command(command));
    }

    pub(super) fn dispatch_actions(&mut self, actions: &Vec<TaggedAction>, key: &Key) -> Result<(), Box<dyn Error>> {
        for action in actions {
            self.dispatch_action(action, key)?;
        }
        Ok(())
    }

    pub(super) fn dispatch_action(&mut self, action: &TaggedAction, key: &Key) -> Result<(), Box<dyn Error>> {
        match &action.action {
            KeymapAction::KeyPressAndRelease(key_press) => self.send_key_press_and_release(key_press),
            KeymapAction::KeyPress(key) => self.send_key(key, super::PRESS),
            KeymapAction::KeyRepeat(key) => self.send_key(key, super::REPEAT),
            KeymapAction::KeyRelease(key) => self.send_key(key, super::RELEASE),
            KeymapAction::Remap(Remap {
                remap,
                timeout,
                timeout_key,
            }) => {
                let set_timeout = self.override_remaps.is_empty();
                self.override_remaps
                    .push(build_override_table(remap, action.exact_match));

                // Set timeout only if this is the first of multiple eligible remaps,
                // so the behaviour is consistent with how current normal keymap override works
                if set_timeout {
                    if let Some(timeout) = timeout {
                        let expiration = Expiration::OneShot(TimeSpec::from_duration(*timeout));
                        // TODO: Consider handling the timer in ActionDispatcher
                        self.override_timer.unset()?;
                        self.override_timer.set(expiration, TimerSetTimeFlags::empty())?;
                        self.override_timeout_key = timeout_key.clone().or_else(|| Some(vec![*key]))
                    }
                }
            }
            KeymapAction::Launch(command) => self.run_command(command.clone()),
            KeymapAction::SetMode(mode) => {
                self.mode = mode.clone();
                println!("mode: {mode}");
            }
            KeymapAction::SetMark(set) => self.mark_set = *set,
            KeymapAction::WithMark(key_press) => self.send_key_press_and_release(&self.with_mark(key_press)),
            KeymapAction::EscapeNextKey(escape_next_key) => self.escape_next_key = *escape_next_key,
            KeymapAction::Sleep(millis) => self.send_action(Action::Delay(Duration::from_millis(*millis))),
            KeymapAction::SetExtraModifiers(keys) => {
                self.extra_modifiers.clear();
                for key in keys {
                    self.extra_modifiers.insert(*key);
                }
            }
        }
        Ok(())
    }

    pub(super) fn send_key_press_and_release(&mut self, key_press: &KeyPress) {
        // Build extra or missing modifiers. Note that only MODIFIER_KEYS are handled
        // because logical modifiers shouldn't make an impact outside xremap.
        let (mut extra_modifiers, mut missing_modifiers) = self.diff_modifiers(&key_press.modifiers);
        extra_modifiers.retain(|key| super::MODIFIER_KEYS.contains(key) && !self.extra_modifiers.contains(key));
        missing_modifiers.retain(|key| super::MODIFIER_KEYS.contains(key));

        // Emulate the modifiers of KeyPress
        self.send_keys(&missing_modifiers, super::PRESS);
        self.send_keys(&extra_modifiers, super::RELEASE);

        // Press the main key
        self.send_key(&key_press.key, super::PRESS);
        self.send_key(&key_press.key, super::RELEASE);

        self.send_action(Action::Delay(self.keypress_delay));

        // Resurrect the original modifiers
        self.send_keys(&extra_modifiers, super::PRESS);
        self.send_action(Action::Delay(self.keypress_delay));
        self.send_keys(&missing_modifiers, super::RELEASE);
    }

    pub(super) fn with_mark(&self, key_press: &KeyPress) -> KeyPress {
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
