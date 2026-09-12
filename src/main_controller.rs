use crate::client::{build_client, WMClient};
use crate::command_runner::CommandRunner;
use crate::main_impl::Desktop;
use log::debug;

pub struct MainController {
    wmclient: WMClient,
    command_runner: CommandRunner,
    allow_launch: bool,
}

impl MainController {
    pub fn new(log_window_changes: bool, allow_launch: bool, desktop: Desktop) -> Self {
        Self {
            wmclient: build_client(log_window_changes, desktop),
            command_runner: CommandRunner::new(allow_launch),
            allow_launch,
        }
    }

    pub fn wmclient<'a>(&'a mut self) -> &'a mut WMClient {
        &mut self.wmclient
    }

    pub fn run_command(&mut self, command: Vec<String>) {
        if !self.allow_launch {
            println!("Launch is not allowed");
            return;
        }

        match self.wmclient.run(&command) {
            Ok(false) => {
                // could not run command, proceed to fork
                self.command_runner.run(command);
            }
            Ok(true) => debug!("Command delegated: {command:?}"),
            Err(e) => {
                debug!("{command:?} failed: {e:?}");
            }
        }
    }

    pub fn show_popup(&mut self, title: &str, msg: Option<&str>) {
        match msg {
            Some(msg) => {
                self.run_command(vec![
                    "notify-send".into(),
                    "-a".into(),
                    "xremap".into(),
                    title.into(),
                    msg.into(),
                ]);
            }
            None => {
                self.run_command(vec!["notify-send".into(), "-a".into(), "xremap".into(), title.into()]);
            }
        };
    }
}
