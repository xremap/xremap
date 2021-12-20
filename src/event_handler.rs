use crate::config::action::Action;
use crate::config::key_press::{KeyPress, Modifier};
use crate::Config;
use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, Key};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;

pub struct EventHandler {
    device: VirtualDevice,
    override_remap: Option<HashMap<KeyPress, Vec<Action>>>,
    shift: bool,
    control: bool,
    alt: bool,
    windows: bool,
}

impl EventHandler {
    pub fn new(device: VirtualDevice) -> EventHandler {
        EventHandler {
            device,
            override_remap: None,
            shift: false,
            control: false,
            alt: false,
            windows: false,
        }
    }

    // Handle EventType::KEY
    pub fn on_event(&mut self, event: InputEvent, config: &Config) -> Result<(), Box<dyn Error>> {
        let mut key = Key::new(event.code());
        // println!("=> {}: {:?}", event.value(), &key);

        // Apply modmap
        for modmap in &config.modmap {
            if let Some(modmap_key) = modmap.remap.get(&key) {
                key = modmap_key.clone();
                break;
            }
        }

        // Apply keymap
        if let Some(modifier) = MODIFIER_KEYS.get(&key.code()) {
            self.update_modifier(modifier, event.value());
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
        // if event.event_type() == EventType::KEY { println!("{}: {:?}", event.value(), Key::new(event.code())); }
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
            shift: self.shift,
            control: self.control,
            alt: self.alt,
            windows: self.windows,
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
                return Some(actions.iter().map(|a| a.clone()).collect());
            }
        }
        None
    }

    fn dispatch_action(&mut self, action: &Action) -> Result<(), Box<dyn Error>> {
        match action {
            Action::KeyPress(key_press) => {
                self.send_modifier(self.shift, key_press.shift, &SHIFT_KEY)?;
                self.send_modifier(self.control, key_press.control, &CONTROL_KEY)?;
                self.send_modifier(self.alt, key_press.alt, &ALT_KEY)?;
                self.send_modifier(self.windows, key_press.windows, &WINDOWS_KEY)?;

                self.send_key(&key_press.key, PRESS)?;
                self.send_key(&key_press.key, RELEASE)?;

                self.send_modifier(key_press.windows, self.windows, &WINDOWS_KEY)?;
                self.send_modifier(key_press.alt, self.alt, &ALT_KEY)?;
                self.send_modifier(key_press.control, self.control, &CONTROL_KEY)?;
                self.send_modifier(key_press.shift, self.shift, &SHIFT_KEY)?;
            }
            Action::Remap(remap) => {
                let mut override_remap: HashMap<KeyPress, Vec<Action>> = HashMap::new();
                for (key_press, actions) in remap.iter() {
                    override_remap.insert(
                        key_press.clone(),
                        actions.iter().map(|a| a.clone()).collect(),
                    );
                }
                self.override_remap = Some(override_remap)
            }
        }
        Ok(())
    }

    fn send_modifier(&mut self, from: bool, to: bool, key: &Key) -> Result<(), Box<dyn Error>> {
        if !from && to {
            self.send_key(key, PRESS)?;
        }
        if from && !to {
            self.send_key(key, RELEASE)?;
        }
        Ok(())
    }

    fn update_modifier(&mut self, modifier: &Modifier, value: i32) {
        match modifier {
            Modifier::Shift => self.shift = is_pressed(value),
            Modifier::Control => self.control = is_pressed(value),
            Modifier::Alt => self.alt = is_pressed(value),
            Modifier::Windows => self.windows = is_pressed(value),
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
    static ref MODIFIER_KEYS: HashMap<u16, Modifier> = vec![
        // Shift
        (Key::KEY_LEFTSHIFT.code(), Modifier::Shift),
        (Key::KEY_RIGHTSHIFT.code(), Modifier::Shift),
        // Control
        (Key::KEY_LEFTCTRL.code(), Modifier::Control),
        (Key::KEY_RIGHTCTRL.code(), Modifier::Control),
        // Alt
        (Key::KEY_LEFTALT.code(), Modifier::Alt),
        (Key::KEY_RIGHTALT.code(), Modifier::Alt),
        // Windows
        (Key::KEY_LEFTMETA.code(), Modifier::Windows),
        (Key::KEY_RIGHTMETA.code(), Modifier::Windows),
    ].into_iter().collect();

    static ref SHIFT_KEY: Key = Key::new(Key::KEY_LEFTSHIFT.code());
    static ref CONTROL_KEY: Key = Key::new(Key::KEY_LEFTCTRL.code());
    static ref ALT_KEY: Key = Key::new(Key::KEY_LEFTALT.code());
    static ref WINDOWS_KEY: Key = Key::new(Key::KEY_LEFTMETA.code());
}
