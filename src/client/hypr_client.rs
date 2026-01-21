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
        if let Ok(win_opt) = HyprClient::get_active() {
            if let Some(win) = win_opt {
                return Some(win.title);
            }
        }
        None
    }

    fn current_application(&mut self) -> Option<String> {
        if let Ok(win_opt) = HyprClient::get_active() {
            if let Some(win) = win_opt {
                return Some(win.class);
            }
        }
        None
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for hyprland")
    }
}
