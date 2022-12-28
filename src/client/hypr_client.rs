use crate::client::Client;
use hyprland::data::blocking::get_active_window;
use serde_json::Value;
pub struct HyprlandClient;

impl HyprlandClient {
    pub fn new() -> HyprlandClient {
        HyprlandClient {}
    }
}

impl Client for HyprlandClient {
    fn supported(&mut self) -> bool {
        true
    }

    fn current_application(&mut self) -> Option<String> {
        if let Ok(win) = get_active_window() {
            let s = serde_json::to_string(&win).ok()?;
            let v: Value = serde_json::from_str(&s).ok()?;
            let app = v["class"].as_str();
            if app.is_some() {
                return Some(String::from(app.unwrap()));
            }
        }
        None
    }
}
