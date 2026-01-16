use futures::executor::block_on;
use log::{debug, warn};
use std::env::temp_dir;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zbus::connection::Builder;
use zbus::{interface, Connection};

use crate::client::Client;

enum Inner {
    Socket(SocketClient),
    Dbus(DbusClient),
}

const KWIN_SCRIPT: &str = include_str!("kwin-script.js");
const KWIN_SCRIPT_PLUGIN_NAME: &str = "xremap";
const KDE_SOCKET_ENV: &str = "KDE_SOCKET";
const DEFAULT_KDE_SOCKET: &str = "/run/xremap/kde.sock";

// Socket client for custom Unix socket communication
struct SocketClient {
    active_window: Arc<Mutex<ActiveWindow>>,
}

impl SocketClient {
    fn new(redact: bool) -> Result<Self, ConnectionError> {
        let socket_path = std::env::var(KDE_SOCKET_ENV)
            .unwrap_or_else(|_| DEFAULT_KDE_SOCKET.to_string());

        debug!("Connecting to KDE socket: {}", socket_path);

        let active_window = Arc::new(Mutex::new(ActiveWindow {
            title: String::new(),
            res_name: String::new(),
            res_class: String::new(),
        }));

        let active_window_clone = Arc::clone(&active_window);

        // Spawn thread to read from socket
        thread::spawn(move || {
            Self::listen_on_socket(&socket_path, active_window_clone, redact);
        });

        Ok(Self { active_window })
    }

    fn listen_on_socket(socket_path: &str, active_window: Arc<Mutex<ActiveWindow>>, redact: bool) {
        loop {
            // Wait for socket to exist
            let path = Path::new(socket_path);
            while !path.exists() {
                thread::sleep(Duration::from_secs(1));
            }

            // Connect to socket
            match UnixStream::connect(socket_path) {
                Ok(stream) => {
                    debug!("Connected to KDE socket");
                    let reader = BufReader::new(&stream);

                    for line in reader.lines() {
                        match line {
                            Ok(text) => {
                                if let Some((caption, class, name)) = Self::parse_window_info(&text) {
                                    let mut aw = active_window.lock().unwrap();
                                    aw.title = caption;
                                    aw.res_class = class;
                                    aw.res_name = name;
                                    // Print in same format as D-Bus version (for readability)
                                    let caption_display = if redact { "[redacted]" } else { &aw.title };
                                    println!("active window: caption: '{}', class: '{}', name: '{}'",
                                             caption_display, aw.res_class, aw.res_name);
                                }
                            }
                            Err(e) => {
                                debug!("Error reading from socket: {:?}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to connect to KDE socket: {:?}, retrying...", e);
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    }

    // Parse JSON format from helper: {"caption": "...", "class": "...", "app_id": "..."}
    fn parse_window_info(text: &str) -> Option<(String, String, String)> {
        // Helper sends JSON
        match serde_json::from_str::<serde_json::Value>(text) {
            Ok(obj) => {
                let caption = obj.get("caption")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let class = obj.get("class")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = obj.get("app_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Some((caption, class, name))
            }
            Err(_) => None,
        }
    }

    fn supported(&self) -> bool {
        true
    }

    fn current_window(&self) -> Option<String> {
        let aw = self.active_window.lock().ok()?;
        if aw.title.is_empty() {
            None
        } else {
            Some(aw.title.clone())
        }
    }

    fn current_application(&self) -> Option<String> {
        let aw = self.active_window.lock().ok()?;
        if aw.res_class.is_empty() {
            None
        } else {
            Some(aw.res_class.clone())
        }
    }
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

impl DbusClient {
    pub fn new(redact: bool) -> Self {
        let active_window = Arc::new(Mutex::new(ActiveWindow {
            title: String::new(),
            res_name: String::new(),
            res_class: String::new(),
        }));
        DbusClient { active_window, redact }
    }

    fn connect(&mut self) -> Result<(), ConnectionError> {
        load_kwin_script()?;

        let active_window = Arc::clone(&self.active_window);
        let redact = self.redact;
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let connect = move || -> Result<Connection, anyhow::Error> {
                let awi = ActiveWindowInterface { active_window, redact };

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

impl Client for DbusClient {
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

// New KdeClient that wraps either SocketClient or DbusClient
pub struct KdeClient {
    inner: Inner,
}

impl KdeClient {
    pub fn new(redact: bool) -> Self {
        let inner = if std::env::var(KDE_SOCKET_ENV).is_ok() {
            debug!("KDE_SOCKET env var set, using socket mode");
            Inner::Socket(SocketClient::new(redact).expect("Failed to create socket client"))
        } else {
            debug!("KDE_SOCKET env var not set, using DBus mode");
            Inner::Dbus(DbusClient::new(redact))
        };

        KdeClient { inner }
    }
}

impl Client for KdeClient {
    fn supported(&mut self) -> bool {
        match &mut self.inner {
            Inner::Socket(client) => client.supported(),
            Inner::Dbus(client) => client.supported(),
        }
    }

    fn current_window(&mut self) -> Option<String> {
        match &mut self.inner {
            Inner::Socket(client) => client.current_window(),
            Inner::Dbus(client) => client.current_window(),
        }
    }

    fn current_application(&mut self) -> Option<String> {
        match &mut self.inner {
            Inner::Socket(client) => client.current_application(),
            Inner::Dbus(client) => client.current_application(),
        }
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

// DBus-based client (original implementation)
struct DbusClient {
    active_window: Arc<Mutex<ActiveWindow>>,
    redact: bool,
}

struct ActiveWindowInterface {
    active_window: Arc<Mutex<ActiveWindow>>,
    redact: bool,
}

#[interface(name = "com.k0kubun.Xremap")]
impl ActiveWindowInterface {
    fn notify_active_window(&mut self, caption: String, res_class: String, res_name: String) {
        // I want to always print this, since it is the only way to know what the resource class of applications is.
        let caption_display = if self.redact { "[redacted]".to_string() } else { caption.clone() };
        println!("active window: caption: '{caption_display}', class: '{res_class}', name: '{res_name}'");
        let mut aw = self.active_window.lock().unwrap();
        aw.title = caption;
        aw.res_class = res_class;
        aw.res_name = res_name;
    }
}
