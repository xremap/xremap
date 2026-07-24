use crate::event::{Event, KeyEvent, KeyValue};
use evdev::KeyCode as Key;

pub trait Plugin {
    const IMPLEMENTED: bool = true;
    fn on_key_event(&mut self, key: Key, value: KeyValue) -> Vec<(Key, KeyValue)>;
}

pub struct NoopPlugin {}

impl Plugin for NoopPlugin {
    // Maybe unneeded way to avoid assembly code, when not implemented.
    const IMPLEMENTED: bool = false;
    fn on_key_event(&mut self, _: Key, _: KeyValue) -> Vec<(Key, KeyValue)> {
        unreachable!()
    }
}

pub fn apply_plugin(plugin: &mut impl Plugin, events: Vec<Event>) -> Vec<Event> {
    events
        .into_iter()
        .flat_map(|event| match event {
            Event::KeyEvent(device, key_event) => plugin
                .on_key_event(key_event.key, key_event.value)
                .into_iter()
                .map(|(key, value)| Event::KeyEvent(device.clone(), KeyEvent::new(key, value)))
                .collect(),
            _ => {
                vec![event]
            }
        })
        .collect()
}
