use crate::client::{Client, WindowInfo};
use anyhow::bail;
use log::debug;
use niri_ipc::socket::Socket;
use niri_ipc::{Action, Request, Response, Window};

/// Client for the Niri scrollable Wayland compositor.
///
/// Communicates with Niri via its IPC socket to determine the currently
/// focused window and application. Requires the NIRI_SOCKET environment
/// variable to be set, which Niri does automatically when running as a session.
///
/// Note: This implementation creates a new socket connection for each query.
/// In the future, a persistent connection might be more efficient.
pub struct NiriClient {
    socket: Option<Socket>,
}

impl NiriClient {
    pub fn new() -> NiriClient {
        NiriClient { socket: None }
    }

    fn get_active_window() -> Option<Window> {
        let mut socket = match Socket::connect() {
            Ok(socket) => socket,
            Err(_) => return None,
        };

        let response = match socket.send(Request::Windows) {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => return None,
            Err(_) => return None,
        };

        match response {
            Response::Windows(windows) => windows.into_iter().find(|w| w.is_focused),
            _ => None,
        }
    }

    fn get_socket<'a>(&'a mut self) -> anyhow::Result<&'a mut Socket> {
        if self.socket.is_none() {
            self.socket = Some(Socket::connect()?);
        }

        let socket = self
            .socket
            .as_mut()
            .ok_or_else(|| anyhow::format_err!("This cannot happen"))?;

        Ok(socket)
    }

    fn send_command<'a>(&'a mut self, action: Action) -> anyhow::Result<()> {
        let response = self
            .get_socket()?
            .send(Request::Action(action))?
            .map_err(|err| anyhow::format_err!("Niri failed: {err:?}"))?;

        match response {
            Response::Handled => Ok(()),
            _ => bail!("Niri sent unexpected response."),
        }
    }

    fn get_windows(&mut self) -> anyhow::Result<Vec<Window>> {
        let windows = self
            .get_socket()?
            .send(Request::Windows)?
            .map_err(|err| anyhow::format_err!("Niri failed: {err:?}"))?;

        match windows {
            Response::Windows(windows) => Ok(windows),
            _ => bail!("Niri sent unexpected response."),
        }
    }
}

impl Client for NiriClient {
    fn supported(&mut self) -> bool {
        let socket_path = match std::env::var("NIRI_SOCKET") {
            Ok(path) => path,
            Err(_) => return false,
        };

        if !std::path::Path::new(&socket_path).exists() {
            return false;
        }

        // Try to actually connect to verify it works
        // This ensures we don't claim support if Niri is not responding
        Socket::connect().is_ok()
    }

    fn current_window(&mut self) -> Option<String> {
        Self::get_active_window().and_then(|win| {
            // Prefer title, but fallback to app_id if title is not available
            win.title.or_else(|| win.app_id.clone())
        })
    }

    fn current_application(&mut self) -> Option<String> {
        Self::get_active_window().and_then(|win| {
            // Prefer app_id, but fallback to window title if app_id is not available
            win.app_id.or_else(|| win.title.clone())
        })
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        bail!("window_list not implemented for NIRI")
    }

    fn close_windows_by_app_class(&mut self, app_class: &str) -> anyhow::Result<()> {
        for window in self.get_windows()? {
            if window.app_id.as_deref() == Some(app_class) {
                debug!("Closing: {:?}", window.title);
                self.send_command(Action::CloseWindow { id: Some(window.id) })?;
            }
        }

        Ok(())
    }
}
