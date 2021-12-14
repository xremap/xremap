use evdev::{EventType, InputEvent, InputEventKind};
use evdev::uinput::VirtualDevice;

pub fn on_event(event: InputEvent, device: &mut VirtualDevice) {
    println!("event: {:?}", event);
    if event.kind() == InputEventKind::Key(evdev::Key::KEY_A) {
        device.emit(&[
            InputEvent::new(EventType::KEY, evdev::Key::KEY_B.code(), event.value())
        ]).unwrap();
    } else {
        device.emit(&[event]).unwrap();
    }
}
