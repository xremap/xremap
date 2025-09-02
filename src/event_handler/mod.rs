mod dispatch;
mod matching;
mod multipurpose;
mod r#override;
mod state;

#[cfg(test)]
mod tests;

use crate::action::Action;
use crate::client::WMClient;

use crate::config::keymap::OverrideEntry;
use crate::config::keymap_action::KeymapAction;
use crate::config::modmap_action::{ModmapAction, MultiPurposeKey, PressReleaseKey};
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue, RelativeEvent};
use crate::Config;
use evdev::KeyCode as Key;
use lazy_static::lazy_static;
use log::debug;
use nix::sys::timerfd::TimerFd;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::{Duration, Instant};

use self::multipurpose::MultiPurposeKeyState;

pub const DISGUISED_EVENT_OFFSETTER: u16 = 59974;

pub const KEY_MATCH_ANY: Key = Key(DISGUISED_EVENT_OFFSETTER + 26);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum RelativeHandling {
    SendOriginal,
    SkipOriginal,
}

pub struct KeyOutcome {
    actions: Vec<Action>,
    relative: RelativeHandling,
}

pub struct EventHandler {
    modifiers: HashSet<Key>,

    extra_modifiers: HashSet<Key>,

    pressed_keys: HashMap<Key, Key>,

    application_client: WMClient,
    application_cache: Option<String>,
    title_cache: Option<String>,

    multi_purpose_keys: HashMap<Key, MultiPurposeKeyState>,

    override_remaps: Vec<HashMap<Key, Vec<OverrideEntry>>>,

    override_timeout_key: Option<Vec<Key>>,

    override_timer: TimerFd,

    mode: String,

    mark_set: bool,

    escape_next_key: bool,

    keypress_delay: Duration,
}

pub struct TaggedAction {
    action: KeymapAction,
    exact_match: bool,
}

pub trait EventEngine {
    fn on_events(&mut self, events: &Vec<Event>, config: &Config) -> Result<Vec<Action>, Box<dyn Error>>;
}

impl EventEngine for EventHandler {
    fn on_events(&mut self, events: &Vec<Event>, config: &Config) -> Result<Vec<Action>, Box<dyn Error>> {
        EventHandler::on_events(self, events, config)
    }
}

impl EventHandler {
    pub fn new(timer: TimerFd, mode: &str, keypress_delay: Duration, application_client: WMClient) -> EventHandler {
        EventHandler {
            modifiers: HashSet::new(),
            extra_modifiers: HashSet::new(),
            pressed_keys: HashMap::new(),
            application_client,
            application_cache: None,
            title_cache: None,
            multi_purpose_keys: HashMap::new(),
            override_remaps: vec![],
            override_timeout_key: None,
            override_timer: timer,
            mode: mode.to_string(),
            mark_set: false,
            escape_next_key: false,
            keypress_delay,
        }
    }

    pub fn on_events(&mut self, events: &Vec<Event>, config: &Config) -> Result<Vec<Action>, Box<dyn Error>> {
        let mut out_actions: Vec<Action> = Vec::new();

        let mut mouse_movement_collection: Vec<RelativeEvent> = Vec::new();
        for event in events {
            match event {
                Event::KeyEvent(device, key_event) => {
                    let outcome = self.on_key(key_event, config, device)?;
                    out_actions.extend(outcome.actions);
                }
                Event::RelativeEvent(device, relative_event) => {
                    out_actions.extend(self.on_relative(
                        relative_event,
                        &mut mouse_movement_collection,
                        config,
                        device,
                    )?);
                }
                Event::OtherEvents(event) => out_actions.push(Action::InputEvent(*event)),
                Event::OverrideTimeout => out_actions.extend(self.timeout_override()?),
            };
        }

        if !mouse_movement_collection.is_empty() {
            out_actions.push(Action::MouseMovementEventCollection(mouse_movement_collection));
        }
        Ok(out_actions)
    }

    fn on_key(
        &mut self,
        event: &KeyEvent,
        config: &Config,
        device: &InputDeviceInfo,
    ) -> Result<KeyOutcome, Box<dyn Error>> {
        self.application_cache = None;
        self.title_cache = None;
        let key = Key::new(event.code());
        debug!("=> {:?}: {:?}", event.phase(), &key);

        let (mut key_values, mut pre_actions): (Vec<(Key, KeyValue)>, Vec<Action>) =
            if let Some(key_action) = self.find_modmap(config, &key, device) {
                self.dispatch_keys(key_action, key, event.phase())?
            } else {
                (vec![(key, event.phase())], Vec::new())
            };
        self.maintain_pressed_keys(key, event.phase(), &mut key_values);
        if !self.multi_purpose_keys.is_empty() {
            key_values = self.flush_timeout_keys(key_values);
        }
        let mut actions: Vec<Action> = Vec::new();
        actions.append(&mut pre_actions);
        let mut relative_handling = RelativeHandling::SkipOriginal;

        for (key, value) in key_values.into_iter() {
            if config.virtual_modifiers.contains(&key) {
                self.update_modifier(key, value);
                continue;
            } else if MODIFIER_KEYS.contains(&key) {
                self.update_modifier(key, value);
            } else if value.is_pressed() {
                if self.escape_next_key {
                    self.escape_next_key = false
                } else {
                    let km1 = self.find_keymap(config, &key, device)?;
                    actions.extend(km1.actions);
                    if let Some(tagged) = km1.tagged {
                        actions.extend(self.dispatch_actions(&tagged, &key)?);
                        continue;
                    }
                    let km_any = self.find_keymap(config, &KEY_MATCH_ANY, device)?;
                    actions.extend(km_any.actions);
                    if let Some(tagged) = km_any.tagged {
                        actions.extend(self.dispatch_actions(&tagged, &KEY_MATCH_ANY)?);
                        continue;
                    }
                }
            }

            if key.code() >= DISGUISED_EVENT_OFFSETTER
                && (key.code(), i32::from(value)) == (event.code(), event.value())
            {
                relative_handling = RelativeHandling::SendOriginal;
                continue;
            }
            actions.push(self.send_key(&key, value));
        }
        Ok(KeyOutcome {
            actions,
            relative: relative_handling,
        })
    }

    fn on_relative(
        &mut self,
        event: &RelativeEvent,
        mouse_movement_collection: &mut Vec<RelativeEvent>,
        config: &Config,
        device: &InputDeviceInfo,
    ) -> Result<Vec<Action>, Box<dyn Error>> {
        let key = match event.value {
            1..=i32::MAX => (event.code * 2) + DISGUISED_EVENT_OFFSETTER,

            i32::MIN..=-1 => (event.code * 2) + 1 + DISGUISED_EVENT_OFFSETTER,

            0 => {
                debug!("This event has a value of zero: {:?}", event);

                (event.code * 2) + DISGUISED_EVENT_OFFSETTER
            }
        };

        let mut actions: Vec<Action> = Vec::new();

        let press_outcome = self.on_key(&KeyEvent::new(Key::new(key), KeyValue::Press), config, device)?;
        actions.extend(press_outcome.actions);
        if matches!(press_outcome.relative, RelativeHandling::SendOriginal) {
            let action = RelativeEvent::new_with(event.code, event.value);
            if event.code <= 2 {
                mouse_movement_collection.push(action);
            } else {
                actions.push(Action::RelativeEvent(action));
            }
        }

        let release_outcome = self.on_key(&KeyEvent::new(Key::new(key), KeyValue::Release), config, device)?;
        actions.extend(release_outcome.actions);

        Ok(actions)
    }

    fn dispatch_keys(
        &mut self,
        key_action: ModmapAction,
        key: Key,
        value: KeyValue,
    ) -> Result<(Vec<(Key, KeyValue)>, Vec<Action>), Box<dyn Error>> {
        let mut actions_out: Vec<Action> = Vec::new();
        let keys = match key_action {
            ModmapAction::Key(modmap_key) => vec![(modmap_key, value)],
            ModmapAction::MultiPurposeKey(MultiPurposeKey {
                held,
                alone,
                alone_timeout,
                free_hold,
            }) => {
                match value {
                    KeyValue::Press => {
                        self.multi_purpose_keys.insert(
                            key,
                            MultiPurposeKeyState {
                                held,
                                alone,
                                alone_timeout_at: if free_hold {
                                    None
                                } else {
                                    Some(Instant::now() + alone_timeout)
                                },
                                held_down: false,
                            },
                        );
                        return Ok((vec![], Vec::new()));
                    }
                    KeyValue::Repeat => {
                        if let Some(state) = self.multi_purpose_keys.get_mut(&key) {
                            return Ok((state.repeat(), Vec::new()));
                        }
                    }
                    KeyValue::Release => {
                        if let Some(state) = self.multi_purpose_keys.remove(&key) {
                            return Ok((state.release(), Vec::new()));
                        }
                    }
                }

                vec![(key, value)]
            }
            ModmapAction::PressReleaseKey(PressReleaseKey {
                skip_key_event,
                press,
                repeat,
                release,
            }) => {
                let actions_to_dispatch = match value {
                    KeyValue::Press => press,
                    KeyValue::Release => release,
                    KeyValue::Repeat => repeat,
                };
                let dispatched = self.dispatch_actions(
                    &actions_to_dispatch
                        .into_iter()
                        .map(|action| TaggedAction {
                            action,
                            exact_match: false,
                        })
                        .collect(),
                    &key,
                )?;
                actions_out.extend(dispatched);

                match skip_key_event {
                    true => vec![],
                    false => vec![(key, value)],
                }
            }
        };
        Ok((keys, actions_out))
    }

    fn flush_timeout_keys(&mut self, key_values: Vec<(Key, KeyValue)>) -> Vec<(Key, KeyValue)> {
        let mut flush = false;
        for (_, value) in key_values.iter() {
            if *value == KeyValue::Press {
                flush = true;
                break;
            }
        }

        if flush {
            let mut flushed: Vec<(Key, KeyValue)> = vec![];
            for (_, state) in self.multi_purpose_keys.iter_mut() {
                flushed.extend(state.force_held());
            }

            let flushed_presses: HashSet<Key> = flushed
                .iter()
                .filter_map(|(k, v)| (*v == KeyValue::Press).then_some(*k))
                .collect();
            let key_values: Vec<(Key, KeyValue)> = key_values
                .into_iter()
                .filter(|(key, value)| !(*value == KeyValue::Press && flushed_presses.contains(key)))
                .collect();

            flushed.extend(key_values);
            flushed
        } else {
            key_values
        }
    }
}

lazy_static! {
    pub static ref MODIFIER_KEYS: [Key; 8] = [
        Key::KEY_LEFTSHIFT,
        Key::KEY_RIGHTSHIFT,
        Key::KEY_LEFTCTRL,
        Key::KEY_RIGHTCTRL,
        Key::KEY_LEFTALT,
        Key::KEY_RIGHTALT,
        Key::KEY_LEFTMETA,
        Key::KEY_RIGHTMETA,
    ];
}
