use fork::{fork, setsid, Fork};
use log::{debug, error};
use nix::sys::signal::{self, sigaction, SaFlags, SigAction, SigHandler, SigSet};
use std::process::{exit, Command, Stdio};

pub struct CommandRunner {
    // Whether we've called a sigaction for spawing commands or not
    sigaction_set: bool,
    allow_launch: bool,
}

impl CommandRunner {
    pub fn new(allow_launch: bool) -> Self {
        Self {
            sigaction_set: false,
            allow_launch,
        }
    }

    pub fn run(&mut self, command: Vec<String>) {
        if !self.allow_launch {
            println!("Launch is not allowed");
            return;
        }

        if !self.sigaction_set {
            // Avoid defunct processes
            let sig_action = SigAction::new(SigHandler::SigDfl, SaFlags::SA_NOCLDWAIT, SigSet::empty());
            unsafe {
                sigaction(signal::SIGCHLD, &sig_action).expect("Failed to register SIGCHLD handler");
            }
            self.sigaction_set = true;
        }

        debug!("Running command: {command:?}");
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
                                error!("Error running command: {e:?}");
                                exit(1);
                            }
                        }
                    }
                    Ok(Fork::Parent(_)) => exit(0),
                    Err(e) => {
                        error!("Error spawning process: {e:?}");
                        exit(1);
                    }
                }
            }
            // Parent should simply continue.
            Ok(Fork::Parent(_)) => (),
            Err(e) => error!("Error spawning process: {e:?}"),
        }
    }
}
