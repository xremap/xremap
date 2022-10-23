use evdev::{InputEvent, EventType, Key, uinput::VirtualDevice};
use log::debug;

use crate::{action::Action, event::KeyEvent};

pub struct ActionDispatcher {
    // Device to emit events
    device: VirtualDevice,
}

impl ActionDispatcher {
    pub fn new(device: VirtualDevice) -> ActionDispatcher {
        ActionDispatcher {
            device,
        }
    }

    pub fn on_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::KeyEvent(key_event) => self.on_key_event(key_event)?,
        }
        Ok(())
    }

    fn on_key_event(&mut self, event: KeyEvent) -> std::io::Result<()> {
        let event = InputEvent::new_now(EventType::KEY, event.code(), event.value());
        self.send_event(event)
    }

    pub fn send_event(&mut self, event: InputEvent) -> std::io::Result<()> {
        if event.event_type() == EventType::KEY {
            debug!("{}: {:?}", event.value(), Key::new(event.code()))
        }
        self.device.emit(&[event])
    }
}
