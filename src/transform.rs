use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, Key};
use std::error::Error;
use crate::Config;

// Handle EventType::KEY
pub fn on_event(event: InputEvent, device: &mut VirtualDevice, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut key = &Key::new(event.code());

    // Perform modmap
    for modmap in &config.modmap {
        if let Some(modmap_key) = modmap.remap.get(&key) {
            key = modmap_key;
            break;
        }
    }

    device.emit(&[InputEvent::new(EventType::KEY, key.code(), event.value())])?;
    Ok(())
}
