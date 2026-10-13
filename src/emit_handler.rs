use crate::event::{Event, KeyEvent};
use crate::event_handler::{MODIFIER_KEYS, PRESS, RELEASE, REPEAT};
use evdev::KeyCode as Key;
use log::warn;

pub struct EmitHandler {
    // Physical modifiers that are down.
    physical_modifiers: Vec<Key>,
}

impl EmitHandler {
    pub fn new() -> EmitHandler {
        EmitHandler {
            physical_modifiers: vec![],
        }
    }

    #[cfg(test)]
    pub fn assert_base_state(&self) {
        assert!(self.physical_modifiers.is_empty());
    }

    pub fn map_output(&mut self, events: Vec<Event>) -> Vec<Event> {
        let mut result = vec![];

        for event in events {
            match event {
                Event::KeyEvent(..) => {
                    result.push(event);
                }
                // Extract the event, that operators have ignored.
                Event::ByPassLocal(device, key_event) => {
                    result.push(Event::KeyEvent(device, key_event));
                }
                Event::ExpmapActions { actions, device, value } => {
                    for key in actions.keys {
                        result.push(Event::KeyEvent(device.clone(), KeyEvent::new(key, value)));
                    }
                }
                Event::RelativeEvent(..) | Event::OtherEvents(..) | Event::OverrideTimeout | Event::Tick => {
                    result.push(event);
                }
            };
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
