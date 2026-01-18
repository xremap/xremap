use crate::client::{Client, WindowInfo};
use anyhow::bail;
use niri_ipc::{socket::Socket, Request, Response};

/// Client for the Niri scrollable Wayland compositor.
///
/// Communicates with Niri via its IPC socket to determine the currently
/// focused window and application. Requires the NIRI_SOCKET environment
/// variable to be set, which Niri does automatically when running as a session.
///
/// Note: This implementation creates a new socket connection for each query.
/// In the future, a persistent connection might be more efficient.
pub struct NiriClient;

impl NiriClient {
    pub fn new() -> NiriClient {
        NiriClient {}
    }

    fn get_active_window() -> Option<niri_ipc::Window> {
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
}
