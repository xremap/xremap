use super::adhoc_script_handler::AdhocScriptHandler;
use anyhow::Result;

pub struct KwinScripts {
    adhoc_script_handler: AdhocScriptHandler,
}

impl KwinScripts {
    pub fn new() -> KwinScripts {
        KwinScripts {
            adhoc_script_handler: AdhocScriptHandler::new(),
        }
    }

    pub fn send_active_window_script_once(&self) -> Result<()> {
        let script = [&COMMON_FUNCTIONS, "notifyActiveWindow(get_active_window());"].join("\n");

        self.adhoc_script_handler.run_script(&script)
    }
}

static COMMON_FUNCTIONS: &str = r#"
function notifyActiveWindow(client) {
    if (!client) {
        // Ignore when there is no active window.
        return;
    }

    callDBus(
        "com.k0kubun.Xremap",
        "/com/k0kubun/Xremap",
        "com.k0kubun.Xremap",
        "NotifyActiveWindow",
        "caption" in client ? client.caption : "",
        "resourceClass" in client ? client.resourceClass : ""
    );
}

function get_active_window() {
  return workspace.activeClient ? workspace.activeClient : workspace.activeWindow;
}
"#;
