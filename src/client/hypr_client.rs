use crate::client::{Client, WindowInfo};
use anyhow::bail;
use hyprland::{data::Client as HyprClient, prelude::*};
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

    fn current_window(&mut self) -> Option<String> {
        if let Ok(Some(win)) = HyprClient::get_active() {
            Some(win.title)
        } else {
            None
        }
    }

    fn current_application(&mut self) -> Option<String> {
        if let Ok(Some(win)) = HyprClient::get_active() {
            Some(win.class)
        } else {
            None
        }
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for hyprland")
    }

    fn close_windows_by_app_class(&mut self, _app_class: &str) -> anyhow::Result<()> {
        todo!()
    }
}
