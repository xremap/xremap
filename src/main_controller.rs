use crate::client::{build_client, WMClient};
use crate::command_runner::CommandRunner;
use log::debug;

pub struct MainController {
    wmclient: WMClient,
    command_runner: CommandRunner,
}

impl MainController {
    pub fn new(log_window_changes: bool) -> Self {
        Self {
            wmclient: build_client(log_window_changes),
            command_runner: CommandRunner::new(),
        }
    }

    pub fn wmclient<'a>(&'a mut self) -> &'a mut WMClient {
        &mut self.wmclient
    }

    pub fn run_command(&mut self, command: Vec<String>) {
        match self.wmclient.run(&command) {
            Ok(false) => {
                // could not run command, proceed to fork
                self.command_runner.run(command);
            }
            Ok(true) => {}
            Err(e) => {
                debug!("{command:?} failed: {e:?}");
            }
        }
    }
}
