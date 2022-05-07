use std::collections::HashMap;

use zbus::{blocking::Connection, zvariant::Value};

use crate::client::Client;

pub struct GnomeClient {
    connection: Option<Connection>,
}

impl GnomeClient {
    pub fn new() -> GnomeClient {
        GnomeClient { connection: None }
    }

    fn connect(&mut self) {
        match Connection::session() {
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
        connection.call_method(
            Some("org.gnome.Shell"),
            "/org/gnome/Shell",
            Some("org.gnome.Shell"),
            "Eval",
            &(code),
        ).map_err(|e| {
            eprintln!(r#"Failed to call Eval in Gnome Shell. This could be due to lack of permissions, or not running in unsafe context.
            Attempting to use SafeIntrospection instead. (https://github.com/wilfredwee/gnome-safe-introspection)
            Original error: {e:?}"#);

            
        })
        .ok()
        .and_then(|message| {
            if let Ok((_actor, json)) = message.body::<(bool, String)>() {
                if let Ok(wm_class) = serde_json::from_str::<String>(&json) {
                    return Some(wm_class);
                }
            }
            return None;
        })
        .or_else(|| {
            let message = connection
            .call_method(
                Some("org.gnome.Shell"),
                "/dev/wxwee/SafeIntrospect",
                Some("dev.wxwee.SafeIntrospect"),
                "GetWindows",
                &(),
            ).map_err(|e| {
                eprintln!("Calling SafeIntrospection failed. Please read the README to troubleshoot.");
                e
            })
            .ok()?;

            let windows = message
            .body::<HashMap<u64, HashMap<String, Value<'_>>>>()
            .map_err(|err| {
                eprintln!("Error deserializing body: {:?}. Message: {message:?}", err);
                err
            })
            .ok()?;

        let focused_window = windows.iter().find(|(_window_id, properties)| {
            properties
                .get("has-focus")
                .map(|val| {
                    if let &Value::Bool(bool_val) = val {
                        bool_val
                    } else {
                        eprintln!("Unexpectedly did not get boolean value from has-focus. Got {val:?} instead.");
                        false
                    }
                })
                .unwrap_or(false)
        });

        let wm_class = focused_window
            .and_then(|(_window_id, properties)| properties.get("wm-class"))
            .and_then(|wm_class| {
                if let Value::Str(wm_class_str) = wm_class {
                    Some(wm_class_str.to_string())
                } else {
                    eprintln!("Unexpectedly did not get string value from wm-class. Got {wm_class:?} instead.");
                    None
                }
            });

            wm_class
        })
    }
}
