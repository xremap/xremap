use evdev::KeyCode as Key;

use super::EventHandler;
use crate::event::KeyValue;

impl EventHandler {
    pub fn maintain_pressed_keys(&mut self, key: Key, value: KeyValue, events: &mut [(Key, KeyValue)]) {
        if events.len() != 1 || value != events[0].1 {
            return;
        }

        let event = events[0];
        if value == KeyValue::Press {
            self.pressed_keys.insert(key, event.0);
        } else {
            if let Some(original_key) = self.pressed_keys.get(&key) {
                events[0].0 = *original_key;
            }
            if value == KeyValue::Release {
                self.pressed_keys.remove(&key);
            }
        }
    }

    pub fn update_modifier(&mut self, key: Key, value: KeyValue) {
        if value == KeyValue::Press {
            self.modifiers.insert(key);
        } else if value == KeyValue::Release {
            self.modifiers.remove(&key);
        }
    }
}
