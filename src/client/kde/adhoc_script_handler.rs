use anyhow::Result;
use futures::executor::block_on;
use std::env::temp_dir;
use std::path::Path;
use zbus::Connection;

pub struct AdhocScriptHandler {}

/// The dbus API is slightly different between plugin and adhoc scripts. Adhoc scripts
/// can only be used for one-off execution. Because it can't be checked if it's still running.
/// Logging and exceptions are not returned. They must be found in the journal.
/// The script isn't unloaded automatically, when it ends execution by it self.
impl AdhocScriptHandler {
    pub fn new() -> Self {
        Self {}
    }

    /// Doesn't wait for the script to finish.
    /// Unloads the script from dbus.
    pub fn run_script(&self, script: &str) -> Result<()> {
        let conn = block_on(Connection::session())?;

        let path = temp_dir().join("xremap-oneoff-script.js");

        std::fs::write(&path, script)?;

        let script_id = self.dbus_load_script(&conn, &path)?;

        self.dbus_run_script(&conn, script_id)?;

        std::fs::remove_file(&path)?;

        // If the script fails, the script is automatically unloaded, so unload
        // will fail. It's the only way to get an indication of failure.
        self.dbus_unload_script(&conn, script_id)
    }

    fn get_dbus_script_path(&self, script_id: i32) -> Result<String> {
        if std::env::var("KDE_SESSION_VERSION")? == "5" {
            Ok(format!("/{script_id}"))
        } else {
            Ok(format!("/Scripting/Script{script_id}"))
        }
    }

    fn dbus_load_script(&self, conn: &Connection, path: &Path) -> Result<i32> {
        let result = block_on(conn.call_method(
            Some("org.kde.KWin"),
            "/Scripting",
            Some("org.kde.kwin.Scripting"),
            "loadScript",
            &path,
        ));

        Ok(result?.body().deserialize::<i32>()?)
    }

    fn dbus_run_script(&self, conn: &Connection, script_id: i32) -> Result<()> {
        block_on(conn.call_method(
            Some("org.kde.KWin"),
            self.get_dbus_script_path(script_id)?,
            Some("org.kde.kwin.Script"),
            "run",
            &(),
        ))?;

        Ok(())
    }

    /// Stopping also unloads the script.
    fn dbus_unload_script(&self, conn: &Connection, script_id: i32) -> Result<()> {
        block_on(conn.call_method(
            Some("org.kde.KWin"),
            self.get_dbus_script_path(script_id)?,
            Some("org.kde.kwin.Script"),
            "stop",
            &(),
        ))?;

        Ok(())
    }
}
