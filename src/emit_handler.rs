use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::event_handler::{MODIFIER_KEYS, PRESS, RELEASE, REPEAT};
use evdev::KeyCode as Key;
use log::warn;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Emit {
    Single(Event),
}

impl Emit {
    pub fn key_release(device: Rc<InputDeviceInfo>, code: Key) -> Emit {
        Emit::Single(Event::KeyEvent(device, KeyEvent::new(code, KeyValue::Release)))
    }
    pub fn key_press(device: Rc<InputDeviceInfo>, code: Key) -> Emit {
        Emit::Single(Event::KeyEvent(device, KeyEvent::new(code, KeyValue::Press)))
    }
    pub fn key_repeat(device: Rc<InputDeviceInfo>, code: Key) -> Emit {
        Emit::Single(Event::KeyEvent(device, KeyEvent::new(code, KeyValue::Repeat)))
    }

    pub fn key_event(device: Rc<InputDeviceInfo>, key_event: KeyEvent) -> Emit {
        Emit::Single(Event::KeyEvent(device, key_event))
    }
}

pub struct EmitHandler {
    // Physical modifiers that are down.
    physical_modifiers: Vec<Key>,
    // Currently emitted modifier keys
    emitted_modifiers: Vec<Key>,
}

impl EmitHandler {
    pub fn new() -> EmitHandler {
        EmitHandler {
            physical_modifiers: vec![],
            emitted_modifiers: vec![],
        }
    }

    #[cfg(test)]
    pub fn assert_base_state(&self) {
        assert!(self.physical_modifiers.is_empty());
        assert!(self.emitted_modifiers.is_empty());
    }

    pub fn map_output(&mut self, events: Vec<Emit>) -> Vec<Event> {
        let mut result = vec![];

        for event in events {
            match event {
                Emit::Single(event) => {
                    let event = match event {
                        // Extract the event, that operators have ignored.
                        Event::ByPassLocal(event) => *event,
                        event => event,
                    };

                    update_modifier_state(&mut self.emitted_modifiers, &event);
                    result.push(event);
                }
            }
        }

        result
    }

    pub fn on_event(&mut self, event: &Event) {
        update_modifier_state(&mut self.physical_modifiers, event)
    }
}

fn update_modifier_state(modifiers: &mut Vec<Key>, event: &Event) {
    if let Event::KeyEvent(_, key_event) = event {
        let key = Key(key_event.code());

        if !MODIFIER_KEYS.contains(&key) {
            return;
        }

        if key_event.value() == PRESS {
            if modifiers.contains(&key) {
                warn!("Pressed key pressed again: {:?}", key);
            } else {
                modifiers.push(key);
            }
        } else if key_event.value() == RELEASE {
            if !modifiers.contains(&key) {
                warn!("Non-pressed key is released: {:?}", key);
            } else {
                modifiers.retain(|&x| x != key);
            }
        } else if key_event.value() == REPEAT {
            if !modifiers.contains(&key) {
                warn!("Non-pressed key is repeated: {:?}", key);
            }
        } else {
            // Ignore invalid
        };
    }
}
