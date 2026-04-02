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

    pub fn to_disguised_key(&self) -> u16 {
        // Because a "full" RELATIVE event is only one event,
        // it doesn't translate very well into a KEY event (because those have a "press" event and an "unpress" event).
        // The solution used here is to send two events for each relative event :
        // one for the press "event" and one for the "unpress" event.

        // All relative events (except maybe those i haven't found information about (REL_DIAL, REL_MISC and REL_RESERVED))
        // can have either a positive value or a negative value.
        // A negative value is associated with a different action than the positive value.
        // Specifically, negative values are associated with the opposite of the action that would emit a positive value.
        // For example, a positive value for a scroll event (REL_WHEEL) comes from an upscroll, while a negative value comes from a downscroll.
        match self.value {
            // Positive and negative values can be really high because the events are relative,
            // so their values are variable, meaning we have to match with all positive/negative values.
            // Not sure if there is any relative event with a fixed value.
            1..=i32::MAX => (self.code * 2) + DISGUISED_EVENT_OFFSETTER,
            // While some events may appear to have a fixed value,
            // events like scrolling will have higher values with more "agressive" scrolling.

            // *2 to create a "gap" between events (since multiplying by two means that all resulting values will be even, the odd numbers between will be missing),
            // +1 if the event has a negative value to "fill" the gap (since adding one shifts the parity from even to odd),
            // and adding DISGUISED_EVENT_OFFSETTER,
            // so that the total as a keycode corresponds to one of the custom aliases that
            // are created in config::key::parse_key specifically for these "disguised" relative events.
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
