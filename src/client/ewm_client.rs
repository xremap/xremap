//
// emacsclient --eval "(ewm-get-window-info-json)"

use crate::client::{Client, WindowInfo};
use anyhow::bail;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct EwmWindowInfo {
    id: Option<u32>,
    app: Option<String>,
    title: Option<String>,
    pid: Option<u32>,
}

pub struct EwmClient;

impl EwmClient {
    pub fn new() -> EwmClient {
        EwmClient {}
    }

    fn get_window_info(&self) -> Option<EwmWindowInfo> {
        println!("json for ewm before...");
        let output = match Command::new("emacsclient")
            .arg("--timeout=3")
            .arg("--eval")
            .arg("(ewm-get-window-info-json)")
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute emacsclient: {} (Is Emacs server running?)", e);
                return None;
            }
        };

        if !output.status.success() {
            println!("emacsclient command failed with status: {:?}", output.status);
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            return None;
        }

        let stdout = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to parse stdout as UTF-8: {}", e);
                return None;
            }
        };
        let trimmed = stdout.trim();

        // emacsclient returns a string like: "\"{...}\""
        // First parse as JSON string to get the inner JSON, then parse that
        let json_str: String = match serde_json::from_str(trimmed) {
            Ok(s) => s,
            Err(e) => {
                println!("Failed to parse outer JSON string: {}, raw output: {}", e, trimmed);
                return None;
            }
        };
        println!("json for ewm {json_str}");

        match serde_json::from_str::<EwmWindowInfo>(&json_str) {
            Ok(info) => Some(info),
            Err(e) => {
                println!("Failed to parse window info JSON: {}", e);
                None
            }
        }
    }
}

impl Client for EwmClient {
    fn supported(&mut self) -> bool {
        // Check if emacsclient can connect to a running Emacs server
        Command::new("emacsclient")
            .arg("--timeout=1")
            .arg("--eval")
            .arg("t")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn current_window(&mut self) -> Option<String> {
        let title = self.get_window_info().and_then(|info| info.title);

        if let Some(title) = &title {
            println!("current_window for ewm {title}");
        }
        title
    }

    fn current_application(&mut self) -> Option<String> {
        let app = self.get_window_info().and_then(|info| info.app);
        if let Some(title) = &app {
            println!("current_app for ewm {title}");
        }
        app
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for ewm")
    }
}
