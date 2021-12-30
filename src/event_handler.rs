use crate::client::{build_client, WMClient};
use crate::command::run_command;
use crate::config::action::Action;
use crate::config::application::Application;
use crate::config::key_action::KeyAction;
use crate::config::key_press::{KeyPress, Modifier};
use crate::Config;
use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, Key};
use lazy_static::lazy_static;
use log::debug;
use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};

pub struct EventHandler {
    device: VirtualDevice,
    wm_client: WMClient,
    multi_purpose_keys: HashMap<Key, MultiPurposeKeyState>,
    override_remap: Option<HashMap<KeyPress, Vec<Action>>>,
    application_cache: Option<String>,
    shift: PressState,
    control: PressState,
    alt: PressState,
    windows: PressState,
}

impl EventHandler {
    pub fn new(device: VirtualDevice) -> EventHandler {
        EventHandler {
            device,
            wm_client: build_client(),
            multi_purpose_keys: HashMap::new(),
            override_remap: None,
            application_cache: None,
            shift: PressState::new(false),
            control: PressState::new(false),
            alt: PressState::new(false),
            windows: PressState::new(false),
        }
    }

    // Handle EventType::KEY
    pub fn on_event(&mut self, event: InputEvent, config: &Config) -> Result<(), Box<dyn Error>> {
        self.application_cache = None; // expire cache
        let key = Key::new(event.code());
        debug!("=> {}: {:?}", event.value(), &key);

        // Apply modmap
        let mut key_values = if let Some(key_action) = self.find_modmap(&config, &key) {
            self.dispatch_keys(key_action, key, event.value())
        } else {
            vec![(key, event.value())]
        };
        if !self.multi_purpose_keys.is_empty() {
            key_values = self.flush_timeout_keys(key_values);
        }

        // Apply keymap
        for (key, value) in key_values.into_iter() {
            let key_press = self.key_to_key_press(&key);
            // Run hotkey commands
            if self.handle_hotkey(&key_press, value, &config) {
                return Ok(());
            } else if MODIFIER_KEYS.contains(&key.code()) {
                self.update_modifier(key.code(), value);
            } else if let Some(actions) = self.find_keymap(config, &key_press, value) {
                for action in &actions {
                    self.dispatch_action(action)?;
                }
                return Ok(());
            }
            self.send_key(&key, value)?;
        }
        Ok(())
    }

    fn handle_hotkey(&mut self, key_press: &KeyPress, value: i32, config: &Config) -> bool {
        let mut hotkey_used = false;
        'outer_hotkey: for hotkey in &config.hotkeys {
            for hotkey_key_press in &hotkey.keys {
                if hotkey_key_press == key_press && is_pressed(value) {
                    if let Some(app) = &hotkey.application {
                        if !self.match_application(app) {
                            continue 'outer_hotkey;
                        }
                    }
                    run_command(hotkey.command.clone());
                    hotkey_used = true;
                    break 'outer_hotkey;
                }
            }
        }
        return hotkey_used;
    }

    pub fn send_event(&mut self, event: InputEvent) -> std::io::Result<()> {
        if event.event_type() == EventType::KEY {
            debug!("{}: {:?}", event.value(), Key::new(event.code()))
        }
        self.device.emit(&[event])
    }

    fn send_key(&mut self, key: &Key, value: i32) -> std::io::Result<()> {
        let event = InputEvent::new(EventType::KEY, key.code(), value);
        self.send_event(event)
    }

    fn dispatch_keys(&mut self, key_action: KeyAction, key: Key, value: i32) -> Vec<(Key, i32)> {
        match key_action {
            KeyAction::Key(modmap_key) => vec![(modmap_key.clone(), value)],
            KeyAction::MultiPurposeKey(multi_purpose_key) => {
                if value == PRESS {
                    self.multi_purpose_keys.insert(
                        key.clone(),
                        MultiPurposeKeyState {
                            held: multi_purpose_key.held,
                            alone: multi_purpose_key.alone,
                            alone_timeout_at: Some(
                                Instant::now() + Duration::from_millis(multi_purpose_key.alone_timeout_millis),
                            ),
                        },
                    );
                    return vec![]; // delay the press
                } else if value == REPEAT {
                    if let Some(state) = self.multi_purpose_keys.get_mut(&key) {
                        return state.repeat();
                    }
                } else if value == RELEASE {
                    if let Some(state) = self.multi_purpose_keys.remove(&key) {
                        return state.release();
                    }
                } else {
                    panic!("unexpected key event value: {}", value);
                }
                // fallthrough on state discrepancy
                vec![(key, value)]
            }
        }
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
            flushed.extend(key_values);
            flushed
        } else {
            key_values
        }
    }

    fn find_modmap(&mut self, config: &Config, key: &Key) -> Option<KeyAction> {
        for modmap in &config.modmap {
            if let Some(key_action) = modmap.remap.get(&key) {
                if let Some(application_matcher) = &modmap.application {
                    if !self.match_application(application_matcher) {
                        continue;
                    }
                }
                return Some(key_action.clone());
            }
        }
        None
    }

    fn key_to_key_press(&mut self, key: &Key) -> KeyPress {
        KeyPress {
            key: key.clone(),
            shift: self.shift.left || self.shift.right,
            control: self.control.left || self.control.right,
            alt: self.alt.left || self.alt.right,
            windows: self.windows.left || self.windows.right,
        }
    }

    fn find_keymap(&mut self, config: &Config, key_press: &KeyPress, value: i32) -> Option<Vec<Action>> {
        if !is_pressed(value) {
            return None;
        }

        if let Some(override_remap) = &self.override_remap {
            let override_remap = override_remap.clone();
            self.override_remap = None;
            if let Some(actions) = override_remap.get(&key_press) {
                return Some(actions.to_vec());
            }
        }
        for keymap in &config.keymap {
            if let Some(actions) = keymap.remap.get(&key_press) {
                if let Some(application_matcher) = &keymap.application {
                    if !self.match_application(application_matcher) {
                        continue;
                    }
                }
                return Some(actions.iter().map(|a| a.clone()).collect());
            }
        }
        None
    }

    fn dispatch_action(&mut self, action: &Action) -> Result<(), Box<dyn Error>> {
        match action {
            Action::KeyPress(key_press) => {
                let next_shift = self.build_state(Modifier::Shift, key_press.shift);
                let next_control = self.build_state(Modifier::Control, key_press.control);
                let next_alt = self.build_state(Modifier::Alt, key_press.alt);
                let next_windows = self.build_state(Modifier::Windows, key_press.windows);

                let prev_shift = self.send_modifier(Modifier::Shift, &next_shift)?;
                let prev_control = self.send_modifier(Modifier::Control, &next_control)?;
                let prev_alt = self.send_modifier(Modifier::Alt, &next_alt)?;
                let prev_windows = self.send_modifier(Modifier::Windows, &next_windows)?;

                self.send_key(&key_press.key, PRESS)?;
                self.send_key(&key_press.key, RELEASE)?;

                self.send_modifier(Modifier::Windows, &prev_windows)?;
                self.send_modifier(Modifier::Alt, &prev_alt)?;
                self.send_modifier(Modifier::Control, &prev_control)?;
                self.send_modifier(Modifier::Shift, &prev_shift)?;
            }
            Action::Remap(remap) => {
                let mut override_remap: HashMap<KeyPress, Vec<Action>> = HashMap::new();
                for (key_press, actions) in remap.iter() {
                    override_remap.insert(key_press.clone(), actions.iter().map(|a| a.clone()).collect());
                }
                self.override_remap = Some(override_remap)
            }
        }
        Ok(())
    }

    fn send_modifier(&mut self, modifier: Modifier, desired: &PressState) -> Result<PressState, Box<dyn Error>> {
        let mut current = match modifier {
            Modifier::Shift => &self.shift,
            Modifier::Control => &self.control,
            Modifier::Alt => &self.alt,
            Modifier::Windows => &self.windows,
        }
        .clone();
        let original = current.clone();
        let left_key = match modifier {
            Modifier::Shift => &SHIFT_KEYS[0],
            Modifier::Control => &CONTROL_KEYS[0],
            Modifier::Alt => &ALT_KEYS[0],
            Modifier::Windows => &WINDOWS_KEYS[0],
        };
        let right_key = match modifier {
            Modifier::Shift => &SHIFT_KEYS[1],
            Modifier::Control => &CONTROL_KEYS[1],
            Modifier::Alt => &ALT_KEYS[1],
            Modifier::Windows => &WINDOWS_KEYS[1],
        };

        if !current.left && desired.left {
            self.send_key(left_key, PRESS)?;
            current.left = true;
        } else if current.left && !desired.left {
            self.send_key(left_key, RELEASE)?;
            current.left = false;
        }

        if !current.right && desired.right {
            self.send_key(right_key, PRESS)?;
            current.right = true;
        } else if current.right && !desired.right {
            self.send_key(right_key, RELEASE)?;
            current.right = false;
        }

        match modifier {
            Modifier::Shift => self.shift = current,
            Modifier::Control => self.control = current,
            Modifier::Alt => self.alt = current,
            Modifier::Windows => self.windows = current,
        };
        Ok(original)
    }

    // Choose a PressState closest to the current state
    fn build_state(&self, modifier: Modifier, pressed: bool) -> PressState {
        let press_state = match modifier {
            Modifier::Shift => &self.shift,
            Modifier::Control => &self.control,
            Modifier::Alt => &self.alt,
            Modifier::Windows => &self.windows,
        };
        if (press_state.left || press_state.right) == pressed {
            press_state.clone() // no change is needed
        } else if pressed {
            // just press left
            PressState {
                left: true,
                right: false,
            }
        } else {
            // release all
            PressState::new(false)
        }
    }

    fn match_application(&mut self, application_matcher: &Application) -> bool {
        // Lazily fill the wm_class cache
        if let None = self.application_cache {
            match self.wm_client.current_application() {
                Some(application) => self.application_cache = Some(application),
                None => self.application_cache = Some(String::new()),
            }
        }

        if let Some(application) = &self.application_cache {
            if let Some(application_only) = &application_matcher.only {
                return application_only.contains(application);
            }
            if let Some(application_not) = &application_matcher.not {
                return !application_not.contains(application);
            }
        }
        false
    }

    fn update_modifier(&mut self, code: u16, value: i32) {
        if code == Key::KEY_LEFTSHIFT.code() {
            self.shift.left = is_pressed(value)
        } else if code == Key::KEY_RIGHTSHIFT.code() {
            self.shift.right = is_pressed(value)
        } else if code == Key::KEY_LEFTCTRL.code() {
            self.control.left = is_pressed(value)
        } else if code == Key::KEY_RIGHTCTRL.code() {
            self.control.right = is_pressed(value)
        } else if code == Key::KEY_LEFTALT.code() {
            self.alt.left = is_pressed(value)
        } else if code == Key::KEY_RIGHTALT.code() {
            self.alt.right = is_pressed(value)
        } else if code == Key::KEY_LEFTMETA.code() {
            self.windows.left = is_pressed(value)
        } else if code == Key::KEY_RIGHTMETA.code() {
            self.windows.right = is_pressed(value)
        } else {
            panic!("unexpected key {:?} at update_modifier", Key::new(code));
        }
    }
}

lazy_static! {
    static ref MODIFIER_KEYS: [u16; 8] = [
        // Shift
        Key::KEY_LEFTSHIFT.code(),
        Key::KEY_RIGHTSHIFT.code(),
        // Control
        Key::KEY_LEFTCTRL.code(),
        Key::KEY_RIGHTCTRL.code(),
        // Alt
        Key::KEY_LEFTALT.code(),
        Key::KEY_RIGHTALT.code(),
        // Windows
        Key::KEY_LEFTMETA.code(),
        Key::KEY_RIGHTMETA.code(),
    ];

    static ref SHIFT_KEYS: [Key; 2] = [
        Key::new(Key::KEY_LEFTSHIFT.code()),
        Key::new(Key::KEY_RIGHTSHIFT.code()),
    ];
    static ref CONTROL_KEYS: [Key; 2] = [
        Key::new(Key::KEY_LEFTCTRL.code()),
        Key::new(Key::KEY_RIGHTCTRL.code()),
    ];
    static ref ALT_KEYS: [Key; 2] = [
        Key::new(Key::KEY_LEFTALT.code()),
        Key::new(Key::KEY_RIGHTALT.code()),
    ];
    static ref WINDOWS_KEYS: [Key; 2] = [
        Key::new(Key::KEY_LEFTMETA.code()),
        Key::new(Key::KEY_RIGHTMETA.code()),
    ];
}

//---

#[derive(Clone)]
struct PressState {
    left: bool,
    right: bool,
}

impl PressState {
    fn new(pressed: bool) -> PressState {
        PressState {
            left: pressed,
            right: pressed,
        }
    }
}

fn is_pressed(value: i32) -> bool {
    value == PRESS || value == REPEAT
}

// InputEvent#value
static RELEASE: i32 = 0;
static PRESS: i32 = 1;
static REPEAT: i32 = 2;

//---

#[derive(Debug)]
struct MultiPurposeKeyState {
    held: Key,
    alone: Key,
    // Some if the first press is still delayed, None if already considered held.
    alone_timeout_at: Option<Instant>,
}

impl MultiPurposeKeyState {
    fn repeat(&mut self) -> Vec<(Key, i32)> {
        if let Some(alone_timeout_at) = &self.alone_timeout_at {
            if Instant::now() < *alone_timeout_at {
                vec![] // still delay the press
            } else {
                self.alone_timeout_at = None; // timeout
                vec![(self.held.clone(), PRESS)]
            }
        } else {
            vec![(self.held.clone(), REPEAT)]
        }
    }

    fn release(&self) -> Vec<(Key, i32)> {
        if let Some(alone_timeout_at) = &self.alone_timeout_at {
            if Instant::now() < *alone_timeout_at {
                // dispatch the delayed press and this release
                vec![(self.alone.clone(), PRESS), (self.alone.clone(), RELEASE)]
            } else {
                // too late. dispatch the held key
                vec![(self.held.clone(), PRESS), (self.held.clone(), RELEASE)]
            }
        } else {
            vec![(self.held.clone(), RELEASE)]
        }
    }

    fn force_held(&mut self) -> Vec<(Key, i32)> {
        if self.alone_timeout_at.is_some() {
            self.alone_timeout_at = None;
            vec![(self.held.clone(), PRESS)]
        } else {
            vec![]
        }
    }
}
