use crate::client::{Client, WindowInfo};
use anyhow::{bail, Result};
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

const KWIN_SCRIPT: &str = include_str!("kwin-script.js");
const KWIN_SCRIPT_PLUGIN_NAME: &str = "xremap";

pub struct KdeClient {
    active_window: Arc<Mutex<ActiveWindow>>,
    log_window_changes: bool,
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

fn dbus_load_script(conn: &Connection, path: &Path) -> Result<i32> {
    Ok(block_on(
        conn.call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "loadScript",
            // since OsStr does not implement zvariant::Type, the temp-path must be valid utf-8
            &(
                path.to_str()
                    .ok_or(anyhow::format_err!("Temp-path must be valid utf-8"))?,
                KWIN_SCRIPT_PLUGIN_NAME,
            ),
        ),
    )?
    .body()
    .deserialize::<i32>()?)
}

fn dbus_unload_script(conn: &Connection) -> Result<bool> {
    Ok(block_on(conn.call_method(
        Some("org.kde.KWin"),
        "/Scripting",
        Some("org.kde.kwin.Scripting"),
        "unloadScript",
        &KWIN_SCRIPT_PLUGIN_NAME,
    ))?
    .body()
    .deserialize::<bool>()?)
}

// Tries both /99 for kde5 and /Scripting/Script99 for kde6
// and squash any errors.
fn dbus_run_script(conn: &Connection, script_obj_id: i32) -> Result<()> {
    for script_obj_path_fn in [|id| format!("/{id}"), |id| format!("/Scripting/Script{id}")] {
        if block_on(conn.call_method(
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
    Err(anyhow::format_err!("Could not start KWIN script, with id: {script_obj_id}"))
}

fn dbus_is_script_loaded(conn: &Connection) -> Result<bool> {
    Ok(block_on(conn.call_method(
        Some("org.kde.KWin"),
        "/Scripting",
        Some("org.kde.kwin.Scripting"),
        "isScriptLoaded",
        &KWIN_SCRIPT_PLUGIN_NAME,
    ))?
    .body()
    .deserialize::<bool>()?)
}

/// Note: Unload is not really usable.
///     This fails: load plugin-script, load adhoc script, unload plugin-script, load plugin-script
///     so it's fragile if other things use adhoc scripts.
fn ensure_script_loaded() -> Result<()> {
    let conn = block_on(Connection::session())?;
    if !dbus_is_script_loaded(&conn)? {
        let init_script = || {
            let temp_file_path = KwinScriptTempFile::new();
            std::fs::write(&temp_file_path.0, KWIN_SCRIPT)?;
            let script_obj_id = dbus_load_script(&conn, &temp_file_path.0)?;
            dbus_run_script(&conn, script_obj_id)?;
            Ok(())
        };
        if let Err(err) = init_script() {
            debug!("Trying to unload kwin-script plugin ('{KWIN_SCRIPT_PLUGIN_NAME}').");
            match dbus_unload_script(&conn, ) {
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
    pub fn new(log_window_changes: bool) -> KdeClient {
        let active_window = Arc::new(Mutex::new(ActiveWindow {
            title: String::new(),
            res_name: String::new(),
            res_class: String::new(),
        }));
        KdeClient {
            active_window,
            log_window_changes,
        }
    }

    fn connect(&mut self) -> Result<()> {
        let active_window = Arc::clone(&self.active_window);
        let log_window_changes = self.log_window_changes;
        let (tx, rx) = channel();

        std::thread::spawn(move || {
            let connect = move || -> Result<Connection> {
                let awi = ActiveWindowInterface {
                    active_window,
                    log_window_changes,
                };

                let connection = Builder::session()?
                    .name("com.k0kubun.Xremap")?
                    .serve_at("/com/k0kubun/Xremap", awi)?
                    .build();

                Ok(block_on(connection)?)
            };

            match connect() {
                Ok(_) => {
                    tx.send(Ok(())).unwrap();
                    loop {
                        thread::sleep(Duration::from_secs(86400));
                    }
                }
                Err(err) => tx.send(Err(err)).unwrap(),
            }
        });

        // Wait for server to start
        rx.recv().unwrap()?;

        // The script sends a message right away, so it's started after the server.
        ensure_script_loaded()?;

        // Busy wait 100ms, so the first use returns a valid value.
        // Testing shows it takes around 10ms to get a response.
        // Notes on correctness:
        //  The lock is just created, so this thread cannot hold the lock already.
        //  `try_lock` doesn't block if the lock is wrongfully held indefinitely by
        //  the other thread, so we are guaranteed to timeout as expected.
        for i in 0..10 {
            if let Ok(aw) = self.active_window.try_lock() {
                if !aw.title.is_empty() {
                    debug!("Connected to KDE within: {}ms", i * 10);
                    return Ok(());
                }
            }

            thread::sleep(Duration::from_millis(10));
        }

        debug!("Connection to KDE was not established within 100ms");

        Ok(())
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

    fn window_list(&mut self) -> Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for KDE")
    }
}

struct ActiveWindow {
    res_class: String,
    res_name: String,
    title: String,
}

struct ActiveWindowInterface {
    active_window: Arc<Mutex<ActiveWindow>>,
    log_window_changes: bool,
}

#[interface(name = "com.k0kubun.Xremap")]
impl ActiveWindowInterface {
    fn notify_active_window(&mut self, caption: String, res_class: String, res_name: String) {
        // Print when log_window_changes is enabled to help identify application resource classes.
        if self.log_window_changes {
            println!("active window: caption: '{caption}', class: '{res_class}', name: '{res_name}'");
        }
        let mut aw = self.active_window.lock().unwrap();
        aw.title = caption;
        aw.res_class = res_class;
        aw.res_name = res_name;
    }
}
