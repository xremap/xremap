use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, Key};
use std::error::Error;
use lazy_static::lazy_static;
use crate::Config;
use crate::config::keypress::Modifier;
use std::collections::HashMap;

pub struct EventHandler {
    pub config: Config,
    pub device: VirtualDevice,
    shift: bool,
    control: bool,
    alt: bool,
    windows: bool,
}

impl EventHandler {
    pub fn new(config: Config, device: VirtualDevice) -> EventHandler {
        EventHandler {
            config,
            device,
            shift: false,
            control: false,
            alt: false,
            windows: false,
        }
    }

    // Handle EventType::KEY
    pub fn on_event(&mut self, event: InputEvent) -> Result<(), Box<dyn Error>> {
        let mut key = Key::new(event.code());

        // Perform modmap
        for modmap in &self.config.modmap {
            if let Some(modmap_key) = modmap.remap.get(&key) {
                key = modmap_key.clone();
                break;
            }
        }

        // Perform keymap
        if let Some(modifier) = MODIFIER_KEYS.get(&key.code()) {
            self.update_modifier(modifier, event.value());
        }

        self.device.emit(&[InputEvent::new(EventType::KEY, key.code(), event.value())])?;
        Ok(())
    }

    pub fn send_event(&mut self, event: InputEvent) -> Result<(), Box<dyn Error>> {
        self.device.emit(&[event])?;
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
// static RELEASE: i32 = 0;
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
}
