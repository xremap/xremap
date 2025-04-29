use crate::client::Client;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use zbus::Connection;

pub struct GnomeClient {
    connection: Option<Connection>,
}

impl GnomeClient {
    pub fn new() -> GnomeClient {
        GnomeClient { connection: None }
    }

    fn connect(&mut self) {
        match block_on(Connection::session()) {
            Ok(connection) => self.connection = Some(connection),
            Err(e) => println!("GnomeClient#connect() failed: {}", e),
        }
    }
}

impl Client for GnomeClient {
    fn supported(&mut self) -> bool {
        self.connect();
        self.current_application().is_some()
    }
    fn current_window(&mut self) -> Option<String> {
        // TODO:  not implemented
        None
    }

    fn current_application(&mut self) -> Option<String> {
        self.connect();
        let connection = match &mut self.connection {
            Some(connection) => connection,
            None => return None,
        };

        // Attempt the latest protocol
        if let Ok(message) = block_on(connection.call_method(
            Some("org.gnome.Shell"),
            "/com/k0kubun/Xremap",
            Some("com.k0kubun.Xremap"),
            "ActiveWindow",
            &(),
        )) {
            if let Ok(json) = message.body().deserialize::<String>() {
                if let Ok(window) = serde_json::from_str::<ActiveWindow>(&json) {
                    return Some(window.wm_class);
                }
            }
        // Fallback to the legacy protocol
        } else if let Ok(message) = block_on(connection.call_method(
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
}

#[derive(Serialize, Deserialize)]
struct ActiveWindow {
    #[serde(default)]
    wm_class: String,
    #[serde(default)]
    title: String,
}
