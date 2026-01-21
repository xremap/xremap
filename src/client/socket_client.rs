use super::socket_monitor::SessionMonitor;
use crate::client::{Client, WindowInfo};
use anyhow::{anyhow, bail, Result};
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

// This client supports a line-based socket protocol where a message is
// sent over the socket as a single line of JSON followed by a newline ('\n'),
// and the response is expected to be a single line of JSON content followed
// by a newline ('\n').
//
// Commands:
// - "ActiveWindow"\n
//   Example response: {"title": "Window Title", "wm_class": "WMClass"}\n
// - {"Run": ["command", "with", "optional", "arguments"]}\n
//   Success response: "Ok"\n
//   Error response (may be arbitrary JSON): "Error message"\n

// XREMAP_SOCKET must generally follow a similar pattern to this default.
// All path components before /{uid}/ or before the socket name, if /{uid}/
// is not in the path, must exist for the client to be supported.
const XREMAP_SOCKET: &str = "/run/xremap/{uid}/xremap.sock";

pub struct SocketClient {
    socket_path: String,
    monitor: Arc<SessionMonitor>,
    _runtime: Runtime,
}

impl SocketClient {
    pub fn new() -> SocketClient {
        let socket_path = std::env::var("XREMAP_SOCKET").unwrap_or(XREMAP_SOCKET.to_string());
        let monitor = Arc::new(SessionMonitor::new(socket_path.clone()));
        let monitor_ = monitor.clone();
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        runtime.spawn(async move { monitor_.run().await });
        SocketClient {
            socket_path,
            monitor,
            _runtime: runtime,
        }
    }

    fn get_active_window(&self) -> Result<ActiveWindow> {
        let json;
        json = self.call_via_socket("ActiveWindow")?;
        Ok(serde_json::from_str::<ActiveWindow>(&json)?)
    }

    fn call_via_socket<T: serde::Serialize>(&self, command: T) -> Result<String> {
        let session = self.monitor.get_active_session().ok_or(anyhow!("no active session"))?;
        let mut stream = UnixStream::connect(session.user_socket)?;
        stream.set_write_timeout(Some(Duration::from_millis(500)))?;
        stream.set_read_timeout(Some(Duration::from_millis(500)))?;
        stream.write_all(serde_json::to_string(&command)?.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        Ok(response)
    }
}

impl Client for SocketClient {
    fn supported(&mut self) -> bool {
        debug!("Using socket path pattern: {}", self.socket_path);
        if let Ok(dynamic_components) = Regex::new(r"/(\{uid\}/.+|[^/{]+)$") {
            let parent_dir = dynamic_components.replace(&self.socket_path, "");
            let path = Path::new(parent_dir.as_ref());
            if path.is_dir() {
                return true;
            }
            println!("Warning: socket directory not found: {}", parent_dir);
        }
        false
    }

    fn current_window(&mut self) -> Option<String> {
        if let Ok(window) = self.get_active_window() {
            return Some(window.title);
        }
        None
    }

    fn current_application(&mut self) -> Option<String> {
        if let Ok(window) = self.get_active_window() {
            return Some(window.wm_class);
        }
        None
    }

    fn run(&mut self, command: &Vec<String>) -> anyhow::Result<bool> {
        let request = serde_json::json!({"Run": command});
        let response = self.call_via_socket(&request)?;
        let parsed = serde_json::from_str::<serde_json::Value>(&response);
        match parsed {
            Ok(v) if v == "Ok" => Ok(true),
            _ => Err(anyhow::format_err!(response)),
        }
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for socket")
    }
}

#[derive(Serialize, Deserialize)]
struct ActiveWindow {
    #[serde(default)]
    wm_class: String,
    #[serde(default)]
    title: String,
}
