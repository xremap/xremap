use crate::client::Client;
use futures::executor::block_on;
use log::debug;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use zbus::{zvariant, Connection, Error, Message};

pub struct GnomeClient {
    socket_path: Option<String>,
    connection: Option<Connection>,
}

impl GnomeClient {
    pub fn new() -> GnomeClient {
        let socket_path = std::env::var("GNOME_SOCKET").ok().filter(|s| !s.is_empty());
        GnomeClient {
            socket_path,
            connection: None,
        }
    }

    fn connect(&mut self) {
        match block_on(Connection::session()) {
            Ok(connection) => self.connection = Some(connection),
            Err(e) => println!("GnomeClient#connect() failed: {}", e),
        }
    }

    fn get_focused_title(&mut self) -> anyhow::Result<String> {
        let window = self.get_active_window()?;
        return Ok(window.title);
    }

    fn get_active_window(&mut self) -> anyhow::Result<ActiveWindow> {
        let json;
        if self.socket_path.is_some() {
            json = self.call_via_socket("ActiveWindow")?;
        } else {
            json = self.call_method("ActiveWindow", &())?.body().deserialize::<String>()?;
        }
        Ok(serde_json::from_str::<ActiveWindow>(&json)?)
    }

    fn call_via_socket<T: serde::Serialize>(&self, command: T) -> anyhow::Result<String> {
        let path = self.socket_path
            .as_ref()
            .ok_or_else(|| anyhow::format_err!("GNOME_SOCKET not set"))?;
        let mut stream = UnixStream::connect(path)?;
        stream.write_all(serde_json::to_string(&command)?.as_bytes())?;
        stream.write_all(b"\n")?;
        stream.flush()?;
        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response)?;
        Ok(response)
    }

    fn call_method<B>(&mut self, method: &str, body: &B) -> anyhow::Result<Message>
    where
        B: serde::ser::Serialize + zvariant::Type,
    {
        self.connect();

        let conn = self
            .connection
            .as_ref()
            .ok_or_else(|| anyhow::format_err!("No gnome connection"))?;

        let result = block_on(conn.call_method(
            Some("org.gnome.Shell"),
            "/com/k0kubun/Xremap",
            Some("com.k0kubun.Xremap"),
            method,
            body,
        ));

        // Try to print some extra information about the failure
        if let Err(Error::MethodError(_, Some(msg), _)) = &result {
            if msg.contains("Object does not exist at path") || msg.contains("No such interface") {
                println!("Error using xremap GNOME extension. Is it installed and enabled?");
            } else if msg.contains("No such method") {
                println!("Error using xremap GNOME extension. Is it updated to the latest version?");
            }
        };

        Ok(result?)
    }
}

impl Client for GnomeClient {
    fn supported(&mut self) -> bool {
        if let Some(socket) = self.socket_path.as_ref() {
            match Path::new(socket).parent() {
                Some(parent) if parent.is_dir() => {
                    debug!("Using GNOME_SOCKET={}", socket);
                    return true;
                }
                _ => {
                    println!("Warning: GNOME_SOCKET={} parent directory not found", socket);
                    return false;
                }
            }
        }

        // Fallback to DBus
        self.connect();
        self.current_application().is_some()
    }

    fn current_window(&mut self) -> Option<String> {
        match self.get_focused_title() {
            Ok(x) => Some(x),
            Err(e) => {
                println!("Error when fetching window title: {e:?}");
                None
            }
        }
    }

    fn current_application(&mut self) -> Option<String> {
        if let Ok(window) = self.get_active_window() {
            return Some(window.wm_class);
        } else if self.socket_path.is_some() {
            // no fallback if GNOME_SOCKET has a value
            return None;
        }

        // Fallback to the legacy protocol
        // self.connect() already called if we got this far
        let connection = match &mut self.connection {
            Some(connection) => connection,
            None => return None,
        };
        if let Ok(message) = block_on(connection.call_method(
            Some("org.gnome.Shell"),
            "/com/k0kubun/Xremap",
            Some("com.k0kubun.Xremap"),
            "WMClass",
            &(),
        )) {
            if let Ok(wm_class) = message.body().deserialize::<String>() {
                return Some(wm_class);
            }
        }
        None
    }

    fn run(&mut self, command: &Vec<String>) -> anyhow::Result<bool> {
        if self.socket_path.is_none() {
            return Ok(false);
        }
        let request = serde_json::json!({"Run": command});
        let response = self.call_via_socket(&request)?;
        let parsed = serde_json::from_str::<serde_json::Value>(&response);
        match parsed {
            Ok(v) if v == "Ok" => Ok(true),
            _ => Err(anyhow::format_err!(response))
        }
    }
}

#[derive(Serialize, Deserialize)]
struct ActiveWindow {
    #[serde(default)]
    wm_class: String,
    #[serde(default)]
    title: String,
}
