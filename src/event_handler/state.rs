use evdev::KeyCode as Key;

use super::EventHandler;

impl EventHandler {
    pub(super) fn maintain_pressed_keys(&mut self, key: Key, value: i32, events: &mut [(Key, i32)]) {
        // Not handling multi-purpose keysfor now; too complicated
        if events.len() != 1 || value != events[0].1 {
            return;
        }

        let event = events[0];
        if value == super::PRESS {
            self.pressed_keys.insert(key, event.0);
        } else {
            if let Some(original_key) = self.pressed_keys.get(&key) {
                events[0].0 = *original_key;
            }
            if value == super::RELEASE {
                self.pressed_keys.remove(&key);
            }
        }
    }

    pub(super) fn update_modifier(&mut self, key: Key, value: i32) {
        if value == super::PRESS {
            self.modifiers.insert(key);
        } else if value == super::RELEASE {
            self.modifiers.remove(&key);
        }
    }
}
