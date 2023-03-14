use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use crate::client::Client;
use zbus::{dbus_interface, fdo, Connection};

pub struct KdeClient {
    active_window: Arc<Mutex<ActiveWindow>>,
}

impl KdeClient {
    pub fn new() -> KdeClient {
        let active_window = Arc::new(Mutex::new(ActiveWindow {
            title: String::new(),
            res_name: String::new(),
            res_class: String::new(),
        }));
        KdeClient { active_window }
    }

    fn connect(&mut self) -> Result<(), ConnectionError> {
        let active_window = Arc::clone(&self.active_window);
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let connect = move || {
                let connection = Connection::new_session().map_err(|_| ConnectionError::Session)?;
                fdo::DBusProxy::new(&connection)
                    .map_err(|_| ConnectionError::Proxy)?
                    .request_name("com.k0kubun.Xremap", fdo::RequestNameFlags::ReplaceExisting.into())
                    .map_err(|_| ConnectionError::RequestName)?;
                let mut object_server = zbus::ObjectServer::new(&connection);
                let mut awi = ActiveWindowInterface { active_window };
                object_server
                    .at(&"/com/k0kubun/Xremap".try_into().unwrap(), awi)
                    .map_err(|_| ConnectionError::Serve)?;
                Ok(object_server)
            };
            let object_server: Result<zbus::ObjectServer, ConnectionError> = connect();
            match object_server {
                Ok(mut object_server) => {
                    tx.send(Ok(()));
                    loop {
                        if let Err(err) = object_server.try_handle_next() {
                            eprintln!("{}", err);
                        }
                    }
                }
                Err(err) => tx.send(Err(err)),
            }
        });
        rx.recv().unwrap()
    }
}

impl Client for KdeClient {
    fn supported(&mut self) -> bool {
        self.connect().is_ok()
    }

    fn current_application(&mut self) -> Option<String> {
        let aw = self.active_window.lock().unwrap();
        Some(aw.res_class.clone())
    }
}

enum ConnectionError {
    Session,
    Proxy,
    RequestName,
    Serve,
}

struct ActiveWindow {
    res_class: String,
    res_name: String,
    title: String,
}

struct ActiveWindowInterface {
    active_window: Arc<Mutex<ActiveWindow>>,
}

#[dbus_interface(name = "com.k0kubun.Xremap")]
impl ActiveWindowInterface {
    fn notify_active_window(&mut self, caption: String, res_class: String, res_name: String) {
        let mut aw = self.active_window.lock().unwrap();
        aw.title = caption;
        aw.res_class = res_class;
        aw.res_name = res_name;
    }
}
