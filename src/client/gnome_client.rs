use crate::client::Client;
use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use zbus::{zvariant, Connection, Error, Message};

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

    fn get_focused_title(&mut self) -> anyhow::Result<String> {
        let json = self.call_method("ActiveWindow", &())?.body().deserialize::<String>()?;

        let window = serde_json::from_str::<ActiveWindow>(&json)?;

        Ok(window.title)
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
