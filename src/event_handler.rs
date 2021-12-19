use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, Key};
use std::error::Error;
use crate::Config;

pub struct EventHandler {
    pub config: Config,
    pub device: VirtualDevice,
}

impl EventHandler {
    // Handle EventType::KEY
    pub fn on_event(&mut self, event: InputEvent) -> Result<(), Box<dyn Error>> {
        let mut key = &Key::new(event.code());

        // Perform modmap
        for modmap in &self.config.modmap {
            if let Some(modmap_key) = modmap.remap.get(&key) {
                key = modmap_key;
                break;
            }
        }

        self.device.emit(&[InputEvent::new(EventType::KEY, key.code(), event.value())])?;
        Ok(())
    }

    pub fn send_event(&mut self, event: InputEvent) -> Result<(), Box<dyn Error>> {
        self.device.emit(&[event])?;
        Ok(())
    }
}
