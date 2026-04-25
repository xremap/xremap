use crate::main_controller::MainController;
use crate::throttle_emit::ThrottleEmit;
use evdev::{uinput::VirtualDevice, EventType, InputEvent, KeyCode as Key};
use log::{debug, error};
use std::thread;

use crate::event::RelativeEvent;
use crate::{action::Action, event::KeyEvent};

pub struct ActionDispatcher {
    // Device to emit events
    device: VirtualDevice,
    // Throttle emitting events
    throttle_emit: Option<ThrottleEmit>,
}

impl ActionDispatcher {
    pub fn new(device: VirtualDevice, throttle_emit: Option<ThrottleEmit>) -> ActionDispatcher {
        ActionDispatcher { device, throttle_emit }
    }

    // Execute Actions created by EventHandler. This should be the only public method of ActionDispatcher.
    pub fn on_action(&mut self, action: Action, mainctrl: &mut MainController) -> anyhow::Result<()> {
        match action {
            Action::KeyEvent(key_event) => self.on_key_event(key_event)?,
            Action::RelativeEvent(relative_event) => self.on_relative_event(relative_event)?,
            Action::MouseMovementEventCollection(mouse_movement_events) => {
                self.send_mousemovement_event_batch(mouse_movement_events)?;
            }
            Action::InputEvent(event) => self.send_event(event)?,
            Action::Command(command) => mainctrl.run_command(command),
            Action::Delay(duration) => thread::sleep(duration),
            Action::CloseByAppClass(app_class) => {
                mainctrl
                    .wmclient()
                    .close_windows_by_app_class(&app_class)
                    .unwrap_or_else(|err| error!("{err:?}"));
            }
        }

        Ok(())
    }

    fn on_key_event(&mut self, event: KeyEvent) -> std::io::Result<()> {
        let event = InputEvent::new_now(EventType::KEY.0, event.code(), event.value());
        self.send_event(event)
    }

    fn on_relative_event(&mut self, event: RelativeEvent) -> std::io::Result<()> {
        let event = InputEvent::new_now(EventType::RELATIVE.0, event.code, event.value);
        self.send_event(event)
    }

    // Send all mouse movement events together, without any synchronization events in between.
    // Mouse movement events need to be sent all at once because they would otherwise be separated by synchronization events.
    // Because sending events one by one through evdev's "emit" function, will add a synchronization event after each event.
    // These artificially added synchronization events would change the semantics of the mouse movement.
    fn send_mousemovement_event_batch(&mut self, eventbatch: Vec<RelativeEvent>) -> std::io::Result<()> {
        let mut mousemovementbatch: Vec<InputEvent> = Vec::new();
        for mouse_movement in eventbatch {
            mousemovementbatch.push(InputEvent::new_now(
                EventType::RELATIVE.0,
                mouse_movement.code,
                mouse_movement.value,
            ));
        }
        self.device.emit(&mousemovementbatch)
    }

    fn send_event(&mut self, event: InputEvent) -> std::io::Result<()> {
        if event.event_type() == EventType::KEY {
            // Throttle
            if let Some(throttle_emit) = &mut self.throttle_emit {
                throttle_emit.sleep_if_needed(Key(event.code()), event.value());
            };

            debug!("{}: {:?}", event.value(), Key::new(event.code()))
        }

        self.device.emit(&[event])
    }
}
