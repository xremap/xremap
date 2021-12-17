use evdev::uinput::VirtualDevice;
use evdev::{EventType, InputEvent, InputEventKind};
use std::error::Error;

pub fn on_event(event: InputEvent, device: &mut VirtualDevice) -> Result<(), Box<dyn Error>> {
    println!("event: {:?}", event);
    if event.kind() == InputEventKind::Key(evdev::Key::KEY_A) {
        device.emit(&[InputEvent::new(
            EventType::KEY,
            evdev::Key::KEY_B.code(),
            event.value(),
        )])?;
    } else {
        device.emit(&[event])?;
    }
    Ok(())
}
