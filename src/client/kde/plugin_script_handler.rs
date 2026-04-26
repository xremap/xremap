use crate::client::kde::kde_client::{KWIN_SCRIPT, KWIN_SCRIPT_PLUGIN_NAME};
use anyhow::Result;
use log::debug;
use std::env::temp_dir;
use std::path::{Path, PathBuf};
use zbus::{block_on, Connection};

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

fn run_script(conn: &Connection, script: &str) -> Result<()> {
    let temp_file_path = KwinScriptTempFile::new();
    std::fs::write(&temp_file_path.0, script)?;
    let script_obj_id = dbus_load_script(&conn, &temp_file_path.0)?;
    dbus_run_script(&conn, script_obj_id)?;
    Ok(())
}

/// Note: Unload is not really usable.
///     This fails: load plugin-script, load adhoc script, unload plugin-script, load plugin-script
///     so it's fragile if other things use adhoc scripts.
pub fn ensure_script_loaded() -> Result<()> {
    let conn = block_on(Connection::session())?;
    if !dbus_is_script_loaded(&conn)? {
        if let Err(err) = run_script(&conn, KWIN_SCRIPT) {
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
