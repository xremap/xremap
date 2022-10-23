use evdev::{Key, InputEvent, EventType};

// Input to EventHandler
#[derive(Debug)]
pub enum Event {
    // InputEvent::Key sent from evdev
    KeyEvent(KeyEvent),
}

#[derive(Debug)]
pub struct KeyEvent {
    key: Key,
    value: KeyValue,
}

#[derive(Debug)]
pub enum KeyValue {
    Press,
    Release,
    Repeat,
}

impl Event {
    // Convert evdev's raw InputEvent to xremap's internal Event
    pub fn new(event: InputEvent) -> Option<Event> {
        let event = match event.event_type() {
            EventType::KEY => Event::KeyEvent(KeyEvent::new(event.code(), event.value())),
            _ => return None,
        };
        Some(event)
    }
}

impl KeyEvent {
    pub fn new(code: u16, value: i32) -> KeyEvent {
        let key = Key::new(code);
        let value = KeyValue::new(value).unwrap();
        KeyEvent { key, value }
    }

    pub fn code(&self) -> u16 {
        self.key.code()
    }

    pub fn value(&self) -> i32 {
        self.value.value()
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
