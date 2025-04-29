use futures::executor::block_on;
use log::{debug, warn};
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zbus::connection::Builder;
use zbus::{interface, Connection};

use crate::client::Client;

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
    fn load_script(&self, path: &Path) -> Result<i32, ConnectionError>;
    fn unload_script(&self) -> Result<bool, ConnectionError>;
    fn start_script(&self, script_obj_id: i32) -> Result<(), ConnectionError>;
    fn is_script_loaded(&self) -> Result<bool, ConnectionError>;
}

impl KWinScripting for Connection {
    fn load_script(&self, path: &Path) -> Result<i32, ConnectionError> {
        block_on(self.call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "loadScript",
            // since OsStr does not implement zvariant::Type, the temp-path must be valid utf-8
            &(path.to_str().ok_or(ConnectionError::TempPathNotValidUtf8)?, KWIN_SCRIPT_PLUGIN_NAME),
        ))
        .map_err(|_| ConnectionError::LoadScriptCall)?
        .body()
        .deserialize::<i32>()
        .map_err(|_| ConnectionError::InvalidLoadScriptResult)
    }

    fn unload_script(&self) -> Result<bool, ConnectionError> {
        block_on(self.call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "unloadScript",
            // since OsStr does not implement zvariant::Type, the temp-path must be valid utf-8
            &KWIN_SCRIPT_PLUGIN_NAME,
        ))
        .map_err(|_| ConnectionError::UnloadScriptCall)?
        .body()
        .deserialize::<bool>()
        .map_err(|_| ConnectionError::InvalidUnloadScriptResult)
    }

    fn start_script(&self, script_obj_id: i32) -> Result<(), ConnectionError> {
        for script_obj_path_fn in [|id| format!("/{id}"), |id| format!("/Scripting/Script{id}")] {
            if block_on(self.call_method(
                Some("org.kde.KWin"),
                script_obj_path_fn(script_obj_id).as_str(),
                Some("org.kde.kwin.Script"),
                "run",
                &(),
            ))
            .is_ok()
            {
                return Ok(());
            }
        }
        Err(ConnectionError::StartScriptCall)
    }

    fn is_script_loaded(&self) -> Result<bool, ConnectionError> {
        block_on(self.call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "isScriptLoaded",
            &KWIN_SCRIPT_PLUGIN_NAME,
        ))
        .map_err(|_| ConnectionError::IsScriptLoadedCall)?
        .body()
        .deserialize::<bool>()
        .map_err(|_| ConnectionError::InvalidIsScriptLoadedResult)
    }
}

fn load_kwin_script() -> Result<(), ConnectionError> {
    let dbus = block_on(Connection::session()).map_err(|_| ConnectionError::ClientSession)?;
    if !dbus.is_script_loaded()? {
        let init_script = || {
            let temp_file_path = KwinScriptTempFile::new();
            std::fs::write(&temp_file_path.0, KWIN_SCRIPT).map_err(|_| ConnectionError::WriteScriptToTempFile)?;
            let script_obj_id = dbus.load_script(&temp_file_path.0)?;
            dbus.start_script(script_obj_id)?;
            Ok(())
        };
        if let Err(err) = init_script() {
            debug!("Trying to unload kwin-script plugin ('{KWIN_SCRIPT_PLUGIN_NAME}').");
            match dbus.unload_script() {
                Err(err) => debug!("Error unloading plugin ('{err:?}'). It may still be loaded and could cause future runs of xremap to fail."),
                Ok(unloaded) if unloaded => debug!("Successfully unloaded plugin."),
                Ok(_) => debug!("Plugin was not loaded in the first place."),
            }
            return Err(err);
        }
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
            let connect = move || -> Result<Connection, anyhow::Error> {
                let awi = ActiveWindowInterface { active_window };

                let connection = Builder::session()?
                    .name("com.k0kubun.Xremap")?
                    .serve_at("/com/k0kubun/Xremap", awi)?
                    .build();

                Ok(block_on(connection)?)
            };

            let result = connect().map_err(|_| ConnectionError::ServerSession);

            match result {
                Ok(_) => {
                    tx.send(Ok(())).unwrap();
                    loop {
                        thread::sleep(Duration::from_secs(86400));
                    }
                }
                Err(err) => tx.send(Err(err)).unwrap(),
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
    fn current_window(&mut self) -> Option<String> {
        let aw = self.active_window.lock().ok()?;
        Some(aw.title.clone())
    }

    fn current_application(&mut self) -> Option<String> {
        let aw = self.active_window.lock().ok()?;
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

    UnloadScriptCall,
    InvalidUnloadScriptResult,

    StartScriptCall,

    IsScriptLoadedCall,
    InvalidIsScriptLoadedResult,

    ServerSession,
}

struct ActiveWindow {
    res_class: String,
    res_name: String,
    title: String,
}

struct ActiveWindowInterface {
    active_window: Arc<Mutex<ActiveWindow>>,
}

#[interface(name = "com.k0kubun.Xremap")]
impl ActiveWindowInterface {
    fn notify_active_window(&mut self, caption: String, res_class: String, res_name: String) {
        // I want to always print this, since it is the only way to know what the resource class of applications is.
        println!("active window: caption: '{caption}', class: '{res_class}', name: '{res_name}'");
        let mut aw = self.active_window.lock().unwrap();
        aw.title = caption;
        aw.res_class = res_class;
        aw.res_name = res_name;
    }
}
