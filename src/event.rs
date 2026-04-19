use crate::device::InputDeviceInfo;
use crate::event_handler::DISGUISED_EVENT_OFFSETTER;
use evdev::{EventType, InputEvent, KeyCode as Key};
use std::rc::Rc;

// Input to EventHandler. This should only contain things that are easily testable.
#[derive(Debug, Clone)]
pub enum Event {
    // InputEvent (EventType::KEY) sent from evdev
    KeyEvent(Rc<InputDeviceInfo>, KeyEvent),
    // InputEvent (EventType::Relative) sent from evdev
    RelativeEvent(Rc<InputDeviceInfo>, RelativeEvent),
    // Any other InputEvent type sent from evdev
    OtherEvents(InputEvent),
    // Timer for nested override reached its timeout
    OverrideTimeout,
    // Ticks for operators
    Tick,
}

impl Event {
    #[cfg(test)]
    pub fn key_release(code: Key) -> Event {
        Event::KeyEvent(crate::tests::get_input_device_info(), KeyEvent::new(code, KeyValue::Release))
    }
    #[cfg(test)]
    pub fn key_press(code: Key) -> Event {
        Event::KeyEvent(crate::tests::get_input_device_info(), KeyEvent::new(code, KeyValue::Press))
    }
    #[cfg(test)]
    pub fn key_repeat(code: Key) -> Event {
        Event::KeyEvent(crate::tests::get_input_device_info(), KeyEvent::new(code, KeyValue::Repeat))
    }
    #[cfg(test)]
    pub fn relative(code: u16, value: i32) -> Event {
        Event::RelativeEvent(crate::tests::get_input_device_info(), RelativeEvent { code, value })
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    value: KeyValue,
}

#[derive(Debug, Clone)]
pub struct RelativeEvent {
    pub code: u16,
    pub value: i32,
}

#[derive(Debug, Copy, Clone)]
pub enum KeyValue {
    Press,
    Release,
    Repeat,
}
impl Event {
    // Convert evdev's raw InputEvent to xremap's internal Event
    pub fn new(device: Rc<InputDeviceInfo>, event: InputEvent) -> Event {
        let event = match event.event_type() {
            EventType::KEY => Event::KeyEvent(device, KeyEvent::new_with(event.code(), event.value())),
            EventType::RELATIVE => Event::RelativeEvent(device, RelativeEvent::new_with(event.code(), event.value())),
            _ => Event::OtherEvents(event),
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

    /// Creates the pseudo keycode from an evdev releative event.
    /// This corresponds to the mapping of pseudo keynames created in `config::key::parse_key`.
    /// Note: There is no sensitivity to the value of the relative event. This means
    ///       that REL_WHEEL will transform into an unpreditable amount of XUPSCROLL
    ///       depending on how the device batches up the events.
    pub fn to_disguised_key(&self) -> u16 {
        // evdev relative events are turned into two events depending on their value
        // being positive or negative. For this reason is the keycode multiplied by two.
        match self.value {
            // Positive values are turned into even numbers
            1..=i32::MAX => (self.code * 2) + DISGUISED_EVENT_OFFSETTER,
            // Negative values are turned into odd numbers
            i32::MIN..=-1 => (self.code * 2) + 1 + DISGUISED_EVENT_OFFSETTER,

            0 => {
                println!("This event has a value of zero : {self:?}");
                // A value of zero would be unexpected for a relative event,
                // since changing something by zero is kinda useless.
                // Just in case it can actually happen (and also because match arms need the same output type),
                // we'll just act like the value of the event was a positive.
                (self.code * 2) + DISGUISED_EVENT_OFFSETTER
            }
        }
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
