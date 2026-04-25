use crate::client::{Client, WindowInfo};
use anyhow::{bail, Result};
use hyprland::data::{Client as HyprClient, Clients};
use hyprland::dispatch::{Dispatch, DispatchType, WindowIdentifier};
use hyprland::prelude::*;
use log::debug;
use std::time::Duration;

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

    fn close_windows_by_app_class(&mut self, app_class: &str) -> Result<()> {
        // Must pick specific windows, because only one is closed at a time.
        for window in Clients::get()? {
            if window.class == app_class {
                debug!("Closing: {:?}", window.title);

                Dispatch::call(DispatchType::CloseWindow(WindowIdentifier::ClassRegularExpression(app_class)))?;

                // Needed otherwise will only the first request be honored.
                std::thread::sleep(Duration::from_millis(20));
            }
        }

        Ok(())
    }
}
