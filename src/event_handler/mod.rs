mod multipurpose;
mod matching;
mod dispatch;
mod state;
mod r#override;

use crate::action::Action;
use crate::client::WMClient;
// OnlyOrNot helpers moved into matching.rs
use crate::config::keymap::{OverrideEntry};
use crate::config::keymap_action::KeymapAction;
use crate::config::modmap_action::{ModmapAction, MultiPurposeKey, PressReleaseKey};
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, RelativeEvent};
use crate::Config;
use evdev::KeyCode as Key;
use lazy_static::lazy_static;
use log::debug;
use nix::sys::timerfd::TimerFd;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::time::{Duration, Instant};

use self::multipurpose::MultiPurposeKeyState;

// This const is a value used to offset RELATIVE events' scancodes
// so that they correspond to the custom aliases created in config::key::parse_key.
// This offset also prevents resulting scancodes from corresponding to non-Xremap scancodes,
// to prevent conflating disguised relative events with other events.
pub const DISGUISED_EVENT_OFFSETTER: u16 = 59974;

// This const is defined a keycode for a configuration key used to match any key.
// It's the offset of XHIRES_LEFTSCROLL + 1
pub const KEY_MATCH_ANY: Key = Key(DISGUISED_EVENT_OFFSETTER + 26);

pub struct EventHandler {
    // Currently pressed modifier keys
    modifiers: HashSet<Key>,
    // Modifiers that are currently pressed but not in the source KeyPress
    extra_modifiers: HashSet<Key>,
    // Make sure the original event is released even if remapping changes while holding the key
    pressed_keys: HashMap<Key, Key>,
    // Check the currently active application
    application_client: WMClient,
    application_cache: Option<String>,
    title_cache: Option<String>,
    // State machine for multi-purpose keys
    multi_purpose_keys: HashMap<Key, MultiPurposeKeyState>,
    // Current nested remaps
    override_remaps: Vec<HashMap<Key, Vec<OverrideEntry>>>,
    // Key triggered on a timeout of nested remaps
    override_timeout_key: Option<Vec<Key>>,
    // Trigger a timeout of nested remaps through select(2)
    override_timer: TimerFd,
    // { set_mode: String }
    mode: String,
    // { set_mark: true }
    mark_set: bool,
    // { escape_next_key: true }
    escape_next_key: bool,
    // keypress_delay_ms
    keypress_delay: Duration,
    // Buffered actions to be dispatched. TODO: Just return actions from each function instead of using this.
    actions: Vec<Action>,
}

struct TaggedAction {
    action: KeymapAction,
    exact_match: bool,
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
            actions: vec![],
        }
    }

    // Handle an Event and return Actions. This should be the only public method of EventHandler.
    pub fn on_events(&mut self, events: &Vec<Event>, config: &Config) -> Result<Vec<Action>, Box<dyn Error>> {
        // a vector to collect mouse movement events to be able to send them all at once as one MouseMovementEventCollection.
        let mut mouse_movement_collection: Vec<RelativeEvent> = Vec::new();
        for event in events {
            match event {
                Event::KeyEvent(device, key_event) => {
                    self.on_key_event(key_event, config, device)?;
                }
                Event::RelativeEvent(device, relative_event) => {
                    self.on_relative_event(relative_event, &mut mouse_movement_collection, config, device)?
                }

                Event::OtherEvents(event) => self.send_action(Action::InputEvent(*event)),
                Event::OverrideTimeout => self.timeout_override()?,
            };
        }
        // if there is at least one mouse movement event, sending all of them as one MouseMovementEventCollection
        if !mouse_movement_collection.is_empty() {
            self.send_action(Action::MouseMovementEventCollection(mouse_movement_collection));
        }
        Ok(self.actions.drain(..).collect())
    }

    // Handle EventType::KEY
    fn on_key_event(
        &mut self,
        event: &KeyEvent,
        config: &Config,
        device: &InputDeviceInfo,
    ) -> Result<bool, Box<dyn Error>> {
        self.application_cache = None; // expire cache
        self.title_cache = None; // expire cache
        let key = Key::new(event.code());
        debug!("=> {}: {:?}", event.value(), &key);

        // Apply modmap
        let mut key_values = if let Some(key_action) = self.find_modmap(config, &key, device) {
            self.dispatch_keys(key_action, key, event.value())?
        } else {
            vec![(key, event.value())]
        };
        self.maintain_pressed_keys(key, event.value(), &mut key_values);
        if !self.multi_purpose_keys.is_empty() {
            key_values = self.flush_timeout_keys(key_values);
        }

        let mut send_original_relative_event = false;
        // Apply keymap
        for (key, value) in key_values.into_iter() {
            if config.virtual_modifiers.contains(&key) {
                self.update_modifier(key, value);
                continue;
            } else if MODIFIER_KEYS.contains(&key) {
                self.update_modifier(key, value);
            } else if is_pressed(value) {
                if self.escape_next_key {
                    self.escape_next_key = false
                } else if let Some(actions) = self.find_keymap(config, &key, device)? {
                    self.dispatch_actions(&actions, &key)?;
                    continue;
                } else if let Some(actions) = self.find_keymap(config, &KEY_MATCH_ANY, device)? {
                    self.dispatch_actions(&actions, &KEY_MATCH_ANY)?;
                    continue;
                }
            }
            // checking if there's a "disguised" key version of a relative event,
            // (scancodes equal to and over DISGUISED_EVENT_OFFSETTER are only "disguised" custom events)
            // and also if it's the same "key" and value as the one that came in.
            if key.code() >= DISGUISED_EVENT_OFFSETTER && (key.code(), value) == (event.code(), event.value()) {
                // if it is, setting send_original_relative_event to true to later tell on_relative_event to send the original event.
                send_original_relative_event = true;
                continue;
            }
            self.send_key(&key, value);
        }

        // Using the Ok() to send a boolean to on_relative_event, which will be used to decide whether to send the original relative event.
        // (True = send the original relative event, false = don't send it.)
        Ok(send_original_relative_event)
    }

    // Handle EventType::RELATIVE
    fn on_relative_event(
        &mut self,
        event: &RelativeEvent,
        mouse_movement_collection: &mut Vec<RelativeEvent>,
        config: &Config,
        device: &InputDeviceInfo,
    ) -> Result<(), Box<dyn Error>> {
        // Because a "full" RELATIVE event is only one event,
        // it doesn't translate very well into a KEY event (because those have a "press" event and an "unpress" event).
        // The solution used here is to send two events for each relative event :
        // one for the press "event" and one for the "unpress" event.

        // These consts are used because 'RELEASE'/'PRESS' are better than '0'/'1' at indicating a button release/press.
        const RELEASE: i32 = 0;
        const PRESS: i32 = 1;

        // All relative events (except maybe those i haven't found information about (REL_DIAL, REL_MISC and REL_RESERVED))
        // can have either a positive value or a negative value.
        // A negative value is associated with a different action than the positive value.
        // Specifically, negative values are associated with the opposite of the action that would emit a positive value.
        // For example, a positive value for a scroll event (REL_WHEEL) comes from an upscroll, while a negative value comes from a downscroll.
        let key = match event.value {
            // Positive and negative values can be really high because the events are relative,
            // so their values are variable, meaning we have to match with all positive/negative values.
            // Not sure if there is any relative event with a fixed value.
            1..=i32::MAX => (event.code * 2) + DISGUISED_EVENT_OFFSETTER,
            // While some events may appear to have a fixed value,
            // events like scrolling will have higher values with more "agressive" scrolling.

            // *2 to create a "gap" between events (since multiplying by two means that all resulting values will be even, the odd numbers between will be missing),
            // +1 if the event has a negative value to "fill" the gap (since adding one shifts the parity from even to odd),
            // and adding DISGUISED_EVENT_OFFSETTER,
            // so that the total as a keycode corresponds to one of the custom aliases that
            // are created in config::key::parse_key specifically for these "disguised" relative events.
            i32::MIN..=-1 => (event.code * 2) + 1 + DISGUISED_EVENT_OFFSETTER,

            0 => {
                println!("This event has a value of zero : {event:?}");
                // A value of zero would be unexpected for a relative event,
                // since changing something by zero is kinda useless.
                // Just in case it can actually happen (and also because match arms need the same output type),
                // we'll just act like the value of the event was a positive.
                (event.code * 2) + DISGUISED_EVENT_OFFSETTER
            }
        };

        // Sending a RELATIVE event "disguised" as a "fake" KEY event press to on_key_event.
        match self.on_key_event(&KeyEvent::new_with(key, PRESS), config, device)? {
            // the boolean value is from a variable at the end of on_key_event from event_handler,
            // used to indicate whether the event got through unchanged.
            true => {
                // Sending the original RELATIVE event if the "press" version of the "fake" KEY event got through on_key_event unchanged.
                let action = RelativeEvent::new_with(event.code, event.value);
                if event.code <= 2 {
                    // If it's a mouse movement event (event.code <= 2),
                    // it is added to mouse_movement_collection to later be sent alongside all other mouse movement event,
                    // as a single MouseMovementEventCollection instead of potentially multiple RelativeEvent .

                    // Mouse movement events need to be sent all at once because they would otherwise be separated by a synchronization event¹,
                    // which the OS handles differently from two unseparated mouse movement events.
                    // For example, a REL_X event², followed by a SYNCHRONIZATION event, followed by a REL_Y event³, followed by a SYNCHRONIZATION event,
                    // will move the mouse cursor by a different amount than a REL_X followed by a REL_Y followed by a SYNCHRONIZATION.

                    // ¹Because Xremap usually sends events one by one through evdev's "emit" function, which adds a synchronization event during each call.
                    // ²Mouse movement along the X (horizontal) axis.
                    // ³Mouse movement along the Y (vertical) axis.
                    mouse_movement_collection.push(action);
                } else {
                    // Otherwise, the event is directly sent as a relative event, to be dispatched like other events.
                    self.send_action(Action::RelativeEvent(action));
                }
            }
            false => {}
        }

        // Sending the "unpressed" version of the "fake" KEY event.
        self.on_key_event(&KeyEvent::new_with(key, RELEASE), config, device)?;

        Ok(())
    }

    // override-related helpers moved to override.rs
    // state helpers moved to state.rs

    fn dispatch_keys(
        &mut self,
        key_action: ModmapAction,
        key: Key,
        value: i32,
    ) -> Result<Vec<(Key, i32)>, Box<dyn Error>> {
        let keys = match key_action {
            ModmapAction::Key(modmap_key) => vec![(modmap_key, value)],
            ModmapAction::MultiPurposeKey(MultiPurposeKey {
                held,
                alone,
                alone_timeout,
                free_hold,
            }) => {
                match value {
                    PRESS => {
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
                        return Ok(vec![]); // delay the press
                    }
                    REPEAT => {
                        if let Some(state) = self.multi_purpose_keys.get_mut(&key) {
                            return Ok(state.repeat());
                        }
                    }
                    RELEASE => {
                        if let Some(state) = self.multi_purpose_keys.remove(&key) {
                            return Ok(state.release());
                        }
                    }
                    _ => panic!("unexpected key event value: {value}"),
                }
                // fallthrough on state discrepancy
                vec![(key, value)]
            }
            ModmapAction::PressReleaseKey(PressReleaseKey {
                skip_key_event,
                press,
                repeat,
                release,
            }) => {
                // Just hook actions, and then emit the original event. We might want to
                // support reordering the key event and dispatched actions later.
                let actions_to_dispatch = match value {
                    PRESS => press,
                    RELEASE => release,
                    _ => repeat,
                };
                self.dispatch_actions(
                    &actions_to_dispatch
                        .into_iter()
                        .map(|action| TaggedAction {
                            action,
                            exact_match: false,
                        })
                        .collect(),
                    &key,
                )?;

                match skip_key_event {
                    true => vec![],              // do not dispatch the original key
                    false => vec![(key, value)], // dispatch the original key
                }
            }
        };
        Ok(keys)
    }

    fn flush_timeout_keys(&mut self, key_values: Vec<(Key, i32)>) -> Vec<(Key, i32)> {
        let mut flush = false;
        for (_, value) in key_values.iter() {
            if *value == PRESS {
                flush = true;
                break;
            }
        }

        if flush {
            let mut flushed: Vec<(Key, i32)> = vec![];
            for (_, state) in self.multi_purpose_keys.iter_mut() {
                flushed.extend(state.force_held());
            }

            // filter out key presses that are part of the flushed events
            let flushed_presses: HashSet<Key> = flushed
                .iter()
                .filter_map(|(k, v)| (*v == PRESS).then_some(*k))
                .collect();
            let key_values: Vec<(Key, i32)> = key_values
                .into_iter()
                .filter(|(key, value)| !(*value == PRESS && flushed_presses.contains(key)))
                .collect();

            flushed.extend(key_values);
            flushed
        } else {
            key_values
        }
    }

}

// with_extra_modifiers and is_remap moved to `matching.rs`

lazy_static! {
    pub(super) static ref MODIFIER_KEYS: [Key; 8] = [
        // Shift
        Key::KEY_LEFTSHIFT,
        Key::KEY_RIGHTSHIFT,
        // Control
        Key::KEY_LEFTCTRL,
        Key::KEY_RIGHTCTRL,
        // Alt
        Key::KEY_LEFTALT,
        Key::KEY_RIGHTALT,
        // Windows
        Key::KEY_LEFTMETA,
        Key::KEY_RIGHTMETA,
    ];
}

// ---

fn is_pressed(value: i32) -> bool {
    value == PRESS || value == REPEAT
}

// InputEvent#value
pub(super) const RELEASE: i32 = 0;
pub(super) const PRESS: i32 = 1;
pub(super) const REPEAT: i32 = 2;
