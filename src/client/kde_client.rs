use log::{info, warn};
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};

use crate::client::Client;
use zbus::{dbus_interface, fdo, Connection};

const KWIN_SCRIPT: &str = include_str!("kwin-script.js");
const KWIN_SCRIPT_PLUGIN_NAME: &str = "xremap";

pub struct KdeClient {
    active_window: Arc<Mutex<ActiveWindow>>,
}

struct KwinScriptTempFile(PathBuf);

impl KwinScriptTempFile {
    fn new() -> Self {
        Self(temp_dir().join("xremap-kwin-script.js"))
    }
}

impl Drop for KwinScriptTempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

trait KWinScripting {
    fn load_script(&self, path: &Path) -> Result<String, ConnectionError>;
    fn start_script(&self, script_obj_path: &str) -> Result<(), ConnectionError>;
    fn is_script_loaded(&self) -> Result<bool, ConnectionError>;
}

impl KWinScripting for Connection {
    fn load_script(&self, path: &Path) -> Result<String, ConnectionError> {
        self.call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "loadScript",
            // since OsStr does not implement zvariant::Type, the temp-path must be valid utf-8
            &(path.to_str().ok_or(ConnectionError::TempPathNotValidUtf8)?, KWIN_SCRIPT_PLUGIN_NAME),
        )
        .map_err(|_| ConnectionError::LoadScriptCall)?
        .body::<i32>()
        .map_err(|_| ConnectionError::InvalidLoadScriptResult)
        .map(|obj_path| format!("/{obj_path}"))
    }

    fn start_script(&self, script_obj_path: &str) -> Result<(), ConnectionError> {
        self.call_method(Some("org.kde.KWin"), script_obj_path, Some("org.kde.kwin.Script"), "run", &())
            .map_err(|_| ConnectionError::StartScriptCall)
            .map(|_| ())
    }

    fn is_script_loaded(&self) -> Result<bool, ConnectionError> {
        self.call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "isScriptLoaded",
            &KWIN_SCRIPT_PLUGIN_NAME,
        )
        .map_err(|_| ConnectionError::IsScriptLoadedCall)?
        .body::<bool>()
        .map_err(|_| ConnectionError::InvalidIsScriptLoadedResult)
    }
}

fn load_kwin_script() -> Result<(), ConnectionError> {
    let dbus = Connection::new_session().map_err(|_| ConnectionError::ClientSession)?;
    if !dbus.is_script_loaded()? {
        let temp_file_path = KwinScriptTempFile::new();
        std::fs::write(&temp_file_path.0, KWIN_SCRIPT).map_err(|_| ConnectionError::WriteScriptToTempFile)?;
        let script_obj_path = dbus.load_script(&temp_file_path.0)?;
        dbus.start_script(&script_obj_path)?;
    }
    Ok(())
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
        load_kwin_script()?;

        let active_window = Arc::clone(&self.active_window);
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let connect = move || {
                let connection = Connection::new_session().map_err(|_| ConnectionError::ServerSession)?;
                fdo::DBusProxy::new(&connection)
                    .map_err(|_| ConnectionError::CreateDBusProxy)?
                    .request_name("com.k0kubun.Xremap", fdo::RequestNameFlags::ReplaceExisting.into())
                    .map_err(|_| ConnectionError::RequestName)?;
                let mut object_server = zbus::ObjectServer::new(&connection);
                let mut awi = ActiveWindowInterface { active_window };
                object_server
                    .at(&"/com/k0kubun/Xremap".try_into().unwrap(), awi)
                    .map_err(|_| ConnectionError::ServeObjServer)?;
                Ok(object_server)
            };
            let object_server: Result<zbus::ObjectServer, ConnectionError> = connect();
            match object_server {
                Ok(mut object_server) => {
                    let _ = tx.send(Ok(()));
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
        let conn_res = self.connect();
        if let Err(err) = &conn_res {
            warn!("Could not connect to kwin-script. Error: {err:?}");
        }
        conn_res.is_ok()
    }

    fn current_application(&mut self) -> Option<String> {
        let aw = self.active_window.lock().unwrap();
        Some(aw.res_class.clone())
    }
}

#[derive(Debug)]
enum ConnectionError {
    TempPathNotValidUtf8,
    WriteScriptToTempFile,
    ClientSession,
    LoadScriptCall,
    InvalidLoadScriptResult,
    StartScriptCall,
    IsScriptLoadedCall,
    InvalidIsScriptLoadedResult,

    ServerSession,
    CreateDBusProxy,
    RequestName,
    ServeObjServer,
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
        // I want to log this on info level, since it is the only way to know what the resource class of applications is.
        info!("active window: caption: '{caption}', class: '{res_class}', name: '{res_name}'");
        let mut aw = self.active_window.lock().unwrap();
        aw.title = caption;
        aw.res_class = res_class;
        aw.res_name = res_name;
    }
}
