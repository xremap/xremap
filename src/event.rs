use evdev::{EventType, InputEvent, Key};

use crate::device::{InputDevice, InputDeviceInfo};

// Input to EventHandler. This should only contain things that are easily testable.
#[derive(Debug)]
pub enum Event<'a> {
    // InputEvent (EventType::KEY) sent from evdev
    KeyEvent(InputDeviceInfo<'a>, KeyEvent),
    // InputEvent (EventType::Relative) sent from evdev
    RelativeEvent(InputDeviceInfo<'a>, RelativeEvent),
    // Any other InputEvent type sent from evdev
    OtherEvents(InputDeviceInfo<'a>, InputEvent),
    // Timer for nested override reached its timeout
    OverrideTimeout,
}

#[derive(Debug)]
pub struct KeyEvent {
    pub key: Key,
    value: KeyValue,
}

#[derive(Debug)]
pub struct RelativeEvent {
    pub code: u16,
    pub value: i32,
}

#[derive(Debug)]
pub enum KeyValue {
    Press,
    Release,
    Repeat,
}
impl<'a> Event<'a> {
    // Convert evdev's raw InputEvent to xremap's internal Event
    pub fn new(device: &InputDevice, event: InputEvent) -> Event {
        let device_info = device.to_info();
        let event = match event.event_type() {
            EventType::KEY => Event::KeyEvent(device_info, KeyEvent::new_with(event.code(), event.value())),
            EventType::RELATIVE => {
                Event::RelativeEvent(device_info, RelativeEvent::new_with(event.code(), event.value()))
            }
            _ => Event::OtherEvents(device_info, event),
        };
        event
    }
}

impl KeyEvent {
    // Constructor with newer interface
    pub fn new(key: Key, value: KeyValue) -> KeyEvent {
        KeyEvent { key, value }
    }

    // Constructor with legacy interface
    pub fn new_with(code: u16, value: i32) -> KeyEvent {
        let key = Key::new(code);
        let value = KeyValue::new(value).unwrap();
        KeyEvent::new(key, value)
    }

    pub fn code(&self) -> u16 {
        self.key.code()
    }

    pub fn value(&self) -> i32 {
        self.value.value()
    }
}

// constructor for relative events.
impl RelativeEvent {
    pub fn new_with(code: u16, value: i32) -> RelativeEvent {
        RelativeEvent { code, value }
    }
}

impl KeyValue {
    fn new(value: i32) -> Option<KeyValue> {
        let event_value = match value {
            0 => KeyValue::Release,
            1 => KeyValue::Press,
            2 => KeyValue::Repeat,
            _ => return None,
        };
        Some(event_value)
    }

    fn value(&self) -> i32 {
        match self {
            KeyValue::Release => 0,
            KeyValue::Press => 1,
            KeyValue::Repeat => 2,
        }
    }
}
