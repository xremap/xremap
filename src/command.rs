use std::process::{Command, Stdio};
use std::thread;
use log::{debug, error};

pub fn run_command(command: Vec<String>) {
    // To avoid defunct processes, spawn a thread to wait on the process.
    thread::spawn(move || {
        debug!("Running command: {:?}", command);
        match Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Err(e) => {
                error!("Error running command: {:?}", e);
            }
            Ok(mut child) => {
                debug!("Process spawned: {:?}, pid {}", command, child.id());
                match child.wait() {
                    Ok(status) => {
                        debug!("Process exited: pid: {}, status: {:?}", child.id(), status);
                    }
                    Err(e) => {
                        error!("Error from process: {:?}", e);
                    }
                }
            }
        }
    });
}
