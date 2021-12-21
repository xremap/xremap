use crate::client::x11_client::X11Client;
use crate::config::action::Action;
use crate::config::key_press::KeyPress;
use crate::config::wm_class::WMClass;
use crate::Config;
use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, Key};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;

pub struct EventHandler {
    device: VirtualDevice,
    x11_client: X11Client,
    override_remap: Option<HashMap<KeyPress, Vec<Action>>>,
    wm_class_cache: Option<String>,
    shift: PressState,
    control: PressState,
    alt: PressState,
    windows: PressState,
}

impl EventHandler {
    pub fn new(device: VirtualDevice) -> EventHandler {
        EventHandler {
            device,
            x11_client: X11Client::new(),
            override_remap: None,
            wm_class_cache: None,
            shift: PressState::new(false),
            control: PressState::new(false),
            alt: PressState::new(false),
            windows: PressState::new(false),
        }
    }

    // Handle EventType::KEY
    pub fn on_event(&mut self, event: InputEvent, config: &Config) -> Result<(), Box<dyn Error>> {
        self.wm_class_cache = None; // expire cache
        let mut key = Key::new(event.code());
        println!("=> {}: {:?}", event.value(), &key);

        // Apply modmap
        for modmap in &config.modmap {
            if let Some(modmap_key) = modmap.remap.get(&key) {
                if let Some(wm_class_matcher) = &modmap.wm_class {
                    if !self.match_wm_class(wm_class_matcher) {
                        continue;
                    }
                }
                key = modmap_key.clone();
                break;
            }
        }

        // Apply keymap
        if MODIFIER_KEYS.contains(&key.code()) {
            self.update_modifier(key.code(), event.value());
        } else if let Some(actions) = self.find_keymap(config, &key, event.value()) {
            for action in &actions {
                self.dispatch_action(action)?;
            }
            return Ok(());
        }

        self.send_key(&key, event.value())?;
        Ok(())
    }

    pub fn send_event(&mut self, event: InputEvent) -> std::io::Result<()> {
        if event.event_type() == EventType::KEY {
            println!("{}: {:?}", event.value(), Key::new(event.code()));
        }
        self.device.emit(&[event])
    }

    fn send_key(&mut self, key: &Key, value: i32) -> std::io::Result<()> {
        let event = InputEvent::new(EventType::KEY, key.code(), value);
        self.send_event(event)
    }

    fn find_keymap(&mut self, config: &Config, key: &Key, value: i32) -> Option<Vec<Action>> {
        if !is_pressed(value) {
            return None;
        }

        let key_press = KeyPress {
            key: key.clone(),
            shift: self.shift.left || self.shift.right,
            control: self.control.left || self.control.right,
            alt: self.alt.left || self.control.right,
            windows: self.windows.left || self.windows.right,
        };
        if let Some(override_remap) = &self.override_remap {
            let override_remap = override_remap.clone();
            self.override_remap = None;
            if let Some(actions) = override_remap.get(&key_press) {
                return Some(actions.to_vec());
            }
        }
        for keymap in &config.keymap {
            if let Some(actions) = keymap.remap.get(&key_press) {
                if let Some(wm_class_matcher) = &keymap.wm_class {
                    if !self.match_wm_class(wm_class_matcher) {
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
            Action::KeyPress(press) => {
                self.send_modifier(
                    &self.shift.clone(),
                    &PressState::new(press.shift),
                    &SHIFT_KEYS,
                )?;
                self.send_modifier(
                    &self.control.clone(),
                    &PressState::new(press.control),
                    &CTRL_KEYS,
                )?;
                self.send_modifier(&self.alt.clone(), &PressState::new(press.alt), &ALT_KEYS)?;
                self.send_modifier(
                    &self.windows.clone(),
                    &PressState::new(press.windows),
                    &WIN_KEYS,
                )?;

                self.send_key(&press.key, PRESS)?;
                self.send_key(&press.key, RELEASE)?;

                self.send_modifier(
                    &PressState::new(press.windows),
                    &self.windows.clone(),
                    &WIN_KEYS,
                )?;
                self.send_modifier(&PressState::new(press.alt), &self.alt.clone(), &ALT_KEYS)?;
                self.send_modifier(
                    &PressState::new(press.control),
                    &self.control.clone(),
                    &CTRL_KEYS,
                )?;
                self.send_modifier(
                    &PressState::new(press.shift),
                    &self.shift.clone(),
                    &SHIFT_KEYS,
                )?;
            }
            Action::Remap(remap) => {
                let mut override_remap: HashMap<KeyPress, Vec<Action>> = HashMap::new();
                for (key_press, actions) in remap.iter() {
                    override_remap
                        .insert(key_press.clone(), actions.iter().map(|a| a.clone()).collect());
                }
                self.override_remap = Some(override_remap)
            }
        }
        Ok(())
    }

    fn send_modifier(
        &mut self,
        from: &PressState,
        to: &PressState,
        keys: &[Key; 2],
    ) -> Result<(), Box<dyn Error>> {
        let left_key = &keys[0];
        if !from.left && to.left {
            self.send_key(left_key, PRESS)?;
        }
        if from.left && !to.left {
            self.send_key(left_key, RELEASE)?;
        }

        let right_key = &keys[1];
        if !from.right && to.right {
            self.send_key(right_key, PRESS)?;
        }
        if from.right && !to.right {
            self.send_key(right_key, RELEASE)?;
        }
        Ok(())
    }

    fn match_wm_class(&mut self, wm_class_matcher: &WMClass) -> bool {
        // Lazily fill the wm_class cache
        if let None = self.wm_class_cache {
            match self.x11_client.current_wm_class() {
                Some(wm_class) => self.wm_class_cache = Some(wm_class),
                None => self.wm_class_cache = Some(String::new()),
            }
        }

        if let Some(wm_class) = &self.wm_class_cache {
            if let Some(wm_class_only) = &wm_class_matcher.only {
                return wm_class_only.contains(wm_class);
            }
            if let Some(wm_class_not) = &wm_class_matcher.not {
                return !wm_class_not.contains(wm_class);
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
    static ref CTRL_KEYS: [Key; 2] = [
        Key::new(Key::KEY_LEFTCTRL.code()),
        Key::new(Key::KEY_RIGHTCTRL.code()),
    ];
    static ref ALT_KEYS: [Key; 2] = [
        Key::new(Key::KEY_LEFTALT.code()),
        Key::new(Key::KEY_RIGHTALT.code()),
    ];
    static ref WIN_KEYS: [Key; 2] = [
        Key::new(Key::KEY_LEFTMETA.code()),
        Key::new(Key::KEY_RIGHTMETA.code()),
    ];
}
