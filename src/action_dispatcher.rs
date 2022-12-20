use std::thread;

use evdev::{uinput::VirtualDevice, EventType, InputEvent, Key};
use fork::{fork, setsid, Fork};
use log::debug;
use log::error;
use nix::sys::signal;
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet};
use std::process::{exit, Command, Stdio};

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
            Action::InputEvent(event) => self.send_event(event)?,
            Action::Command(command) => self.run_command(command),
            Action::Delay(duration) => thread::sleep(duration),
        }
        Ok(())
    }

    fn on_key_event(&mut self, event: KeyEvent) -> std::io::Result<()> {
        let event = InputEvent::new_now(EventType::KEY, event.code(), event.value());
        self.send_event(event)
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
