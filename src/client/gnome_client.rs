use crate::client::Client;
use zbus::Connection;

pub struct GnomeClient {
    connection: Option<Connection>,
}

impl GnomeClient {
    pub fn new() -> GnomeClient {
        GnomeClient { connection: None }
    }

    fn connect(&mut self) {
        match Connection::new_session() {
            Ok(connection) => self.connection = Some(connection),
            Err(e) => println!("GnomeClient#connect() failed: {}", e),
        }
    }
}

impl Client for GnomeClient {
    fn supported(&mut self) -> bool {
        self.connect();
        self.connection.is_some()
    }

    fn current_application(&mut self) -> Option<String> {
        self.connect();
        let connection = match &mut self.connection {
            Some(connection) => connection,
            None => return None,
        };

        let code = "
            const actor = global.get_window_actors().find(a=>a.meta_window.has_focus()===true)
            actor && actor.get_meta_window().get_wm_class()
        ";
        if let Ok(message) = connection.call_method(
            Some("org.gnome.Shell"),
            "/org/gnome/Shell",
            Some("org.gnome.Shell"),
            "Eval",
            &(code),
        ) {
            if let Ok((_actor, json)) = message.body::<(bool, String)>() {
                if let Ok(wm_class) = serde_json::from_str::<String>(&json) {
                    return Some(wm_class);
                }
            }
        }
        None
    }
}
