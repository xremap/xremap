use std::thread;

use evdev::{uinput::VirtualDevice, EventType, InputEvent, KeyCode as Key};
use fork::{fork, setsid, Fork};
use log::debug;
use log::error;
use nix::sys::signal;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet};
use std::process::{exit, Command, Stdio};

use crate::event::RelativeEvent;
use crate::{action::Action, event::KeyEvent};

pub struct ActionDispatcher {
    // Device to emit events
    device: VirtualDevice,
    // Whether we've called a sigaction for spawing commands or not
    sigaction_set: bool,
}

impl ActionDispatcher {
    pub fn new(device: VirtualDevice) -> ActionDispatcher {
        ActionDispatcher {
            device,
            sigaction_set: false,
        }
    }

    // Execute Actions created by EventHandler. This should be the only public method of ActionDispatcher.
    pub fn on_action(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::KeyEvent(key_event) => self.on_key_event(key_event)?,
            Action::RelativeEvent(relative_event) => self.on_relative_event(relative_event)?,
            Action::MouseMovementEventCollection(mouse_movement_events) => {
                // Sending all mouse movement events at once, unseparated by synchronization events.
                self.send_mousemovement_event_batch(mouse_movement_events)?;

                // Mouse movement events need to be sent all at once because they would otherwise be separated by a synchronization event¹,
                // which the OS handles differently from two unseparated mouse movement events.
                // For example,
                // a REL_X event², followed by a SYNCHRONIZATION event, followed by a REL_Y event³, followed by a SYNCHRONIZATION event,
                // will move the mouse cursor by a different amount than
                // a REL_X event followed by a REL_Y event followed by a SYNCHRONIZATION event.

                // ¹Because Xremap usually sends events one by one through evdev's "emit" function, which adds a synchronization event during each call.
                // ²Mouse movement along the X (horizontal) axis.
                // ³Mouse movement along the Y (vertical) axis.
            }

            Action::InputEvent(event) => self.send_event(event)?,
            Action::Command(command) => self.run_command(command),
            Action::Delay(duration) => thread::sleep(duration),
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

    // a function that takes mouse movement events to send in a single batch, unseparated by synchronization events.
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
            debug!("{}: {:?}", event.value(), Key::new(event.code()))
        }
        self.device.emit(&[event])
    }

    fn run_command(&mut self, command: Vec<String>) {
        if !self.sigaction_set {
            // Avoid defunct processes
            let sig_action = SigAction::new(SigHandler::SigDfl, SaFlags::SA_NOCLDWAIT, SigSet::empty());
            unsafe {
                sigaction(signal::SIGCHLD, &sig_action).expect("Failed to register SIGCHLD handler");
            }
            self.sigaction_set = true;
        }

        debug!("Running command: {:?}", command);
        match fork() {
            Ok(Fork::Child) => {
                // Child process should fork again, and the parent should exit 0, while the child
                // should spawn the user command then exit as well.
                match fork() {
                    Ok(Fork::Child) => {
                        setsid().expect("Failed to setsid.");
                        match Command::new(&command[0])
                            .args(&command[1..])
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn()
                        {
                            Ok(child) => {
                                debug!("Process started: {:?}, pid {}", command, child.id());
                                exit(0);
                            }
                            Err(e) => {
                                error!("Error running command: {:?}", e);
                                exit(1);
                            }
                        }
                    }
                    Ok(Fork::Parent(_)) => exit(0),
                    Err(e) => {
                        error!("Error spawning process: {:?}", e);
                        exit(1);
                    }
                }
            }
            // Parent should simply continue.
            Ok(Fork::Parent(_)) => (),
            Err(e) => error!("Error spawning process: {:?}", e),
        }
    }
}
