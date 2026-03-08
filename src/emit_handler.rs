use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue};
use crate::event_handler::{MODIFIER_KEYS, PRESS, RELEASE, REPEAT};
use evdev::KeyCode as Key;
use log::warn;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct EmitCombo {
    pub key: Key,
    pub modifiers: Vec<Key>,
}

#[derive(Debug, Clone)]
pub enum Emit {
    Single(Event),
    #[allow(dead_code)]
    KeyComboWithHold(Rc<InputDeviceInfo>, EmitCombo),
    #[allow(dead_code)]
    SyncModidiers(Rc<InputDeviceInfo>),
}

impl Emit {
    pub fn key_event(device: Rc<InputDeviceInfo>, key_event: KeyEvent) -> Emit {
        Emit::Single(Event::KeyEvent(device, key_event))
    }
}

pub struct EmitHandler {
    // Physical modifiers that are down.
    physical_modifiers: Vec<Key>,
    // Currently emitted modifier keys
    // A subset of the physical modifiers.
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
    pub fn assert_emitted_modifiers_are_synced(&self) {
        assert!(self
            .sync_emitted_modifiers(crate::tests::get_input_device_info(), &self.physical_modifiers)
            .is_empty());
    }

    pub fn map_output(&mut self, events: Vec<Emit>) -> Vec<Event> {
        let mut result = vec![];

        for event in events {
            match event {
                Emit::Single(event) => {
                    update_modifier_state(&mut self.emitted_modifiers, &event);
                    result.push(event);
                }
                Emit::KeyComboWithHold(device, key_combo) => {
                    let mut events = self.sync_emitted_modifiers(device.clone(), &key_combo.modifiers);

                    self.emitted_modifiers = key_combo.modifiers.clone();

                    assert!(!MODIFIER_KEYS.contains(&key_combo.key));

                    events.extend(vec![
                        Event::KeyEvent(device.clone(), KeyEvent::new(key_combo.key, KeyValue::Press)),
                        Event::KeyEvent(device, KeyEvent::new(key_combo.key, KeyValue::Release)),
                    ]);

                    result.extend(events);
                }
                Emit::SyncModidiers(device) => {
                    let events = self.sync_emitted_modifiers(device, &self.physical_modifiers);

                    self.emitted_modifiers = self.physical_modifiers.clone();

                    result.extend(events);
                }
            }
        }

        result
    }

    pub fn on_event(&mut self, event: &Event) {
        update_modifier_state(&mut self.physical_modifiers, event)
    }

    fn sync_emitted_modifiers(&self, device: Rc<InputDeviceInfo>, modifiers: &[Key]) -> Vec<Event> {
        let modifiers_to_release: Vec<Event> = self
            .emitted_modifiers
            .iter()
            .filter_map(|key| {
                if modifiers.contains(key) {
                    None
                } else {
                    // Emitted modifier that should not be pressed.
                    Some(Event::KeyEvent(device.clone(), KeyEvent::new(*key, KeyValue::Release)))
                }
            })
            .collect();

        let modifiers_to_press: Vec<Event> = modifiers
            .iter()
            .filter_map(|key| {
                if self.emitted_modifiers.contains(key) {
                    None
                } else {
                    // Modifier that hasn't been emitted yet.
                    Some(Event::KeyEvent(device.clone(), KeyEvent::new(*key, KeyValue::Press)))
                }
            })
            .collect();

        let mut events = modifiers_to_release;

        events.extend(modifiers_to_press);

        events
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
