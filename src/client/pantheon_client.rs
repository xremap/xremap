use crate::client::{Client, WindowInfo};
use anyhow::{bail, Result};
use std::collections::HashMap;
use zbus::zvariant::Value;
use zbus::{block_on, Connection};

#[derive(Debug)]
struct PantheonWindow {
    id: String,
    app_class: Option<String>,
    title: Option<String>,
    focused: bool,
}

pub struct PantheonClient {
    connection: Option<Connection>,
}

impl PantheonClient {
    pub fn new() -> Self {
        Self { connection: None }
    }

    fn connect(&mut self) -> Result<&mut Connection> {
        if self.connection.is_none() {
            self.connection = Some(block_on(Connection::session())?);
        }

        Ok(self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow::format_err!("This cannot happen"))?)
    }

    fn get_focused_window(&mut self) -> Result<Option<PantheonWindow>> {
        Ok(self.get_windows()?.into_iter().find(|window| window.focused))
    }

    fn get_windows(&mut self) -> Result<Vec<PantheonWindow>> {
        let body = block_on(self.connect()?.call_method(
            Some("org.pantheon.gala"),
            "/org/pantheon/gala/DesktopInterface",
            Some("org.pantheon.gala.DesktopIntegration"),
            "GetWindows",
            &(),
        ))?
        .body();

        Ok(body
            .deserialize::<Vec<(u64, HashMap<String, Value>)>>()?
            .into_iter()
            .map(|(id, dict)| PantheonWindow {
                id: format!("{id}"),
                title: dict.get("title").map(|v| String::try_from(v).unwrap_or_default()),
                app_class: dict.get("wm-class").map(|v| String::try_from(v).unwrap_or_default()),
                focused: dict
                    .get("has-focus")
                    .map(|v| bool::try_from(v))
                    .unwrap_or(Ok(false))
                    .unwrap_or_default(),
            })
            .collect())
    }
}

impl Client for PantheonClient {
    fn supported(&mut self) -> bool {
        self.get_windows().is_ok()
    }

    fn current_window(&mut self) -> Option<String> {
        match self.get_focused_window() {
            Ok(window) => window.and_then(|window| window.title),
            Err(e) => {
                eprintln!("Error when fetching window title: {e:?}");
                None
            }
        }
    }

    fn current_application(&mut self) -> Option<String> {
        match self.get_focused_window() {
            Ok(window) => window.and_then(|window| window.app_class),
            Err(e) => {
                eprintln!("Error when fetching app_class: {e:?}");
                None
            }
        }
    }

    fn window_list(&mut self) -> Result<Vec<WindowInfo>> {
        Ok(self
            .get_windows()?
            .into_iter()
            .map(|window| WindowInfo {
                winid: Some(window.id),
                app_class: window.app_class,
                title: window.title,
            })
            .collect())
    }

    fn close_windows_by_app_class(&mut self, _app_class: &str) -> Result<()> {
        bail!("close_windows_by_app_class not implemented for Pantheon")
    }
}
