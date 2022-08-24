use crate::client::Client;
use std::env;
use x11rb::protocol::xproto::{self};
use x11rb::protocol::xproto::{AtomEnum, Window};
use x11rb::{protocol::xproto::get_property, rust_connection::RustConnection};

pub struct X11Client {
    connection: Option<RustConnection>,
}

impl X11Client {
    pub fn new() -> X11Client {
        X11Client { connection: None }
    }

    fn connect(&mut self) {
        if self.connection.is_some() {
            return;
        }

        if let Err(env::VarError::NotPresent) = env::var("DISPLAY") {
            println!("$DISPLAY is not set. Defaulting to DISPLAY=:0");
            env::set_var("DISPLAY", ":0");
        }
        match x11rb::connect(None) {
            Ok((connection, _)) => self.connection = Some(connection),
            Err(error) => {
                let var = env::var("DISPLAY").unwrap();
                println!("warning: Failed to connect to X11: {error}");
                println!("If you saw \"No protocol specified\", try running `xhost +SI:localuser:root`.");
                println!("If not, make sure `echo $DISPLAY` outputs xremap's $DISPLAY ({var}).");
            }
        }
    }
}

impl Client for X11Client {
    fn supported(&mut self) -> bool {
        self.connect();
        return self.connection.is_some();
        // TODO: Test XGetInputFocus and focused_window > 0?
    }

    fn current_application(&mut self) -> Option<String> {
        self.connect();
        if let Some(conn) = &self.connection {
            let mut window = get_focus_window(conn)?;
            loop {
                if let Some(wm_class) = get_wm_class(conn, window) {
                    // Workaround: https://github.com/JetBrains/jdk8u_jdk/blob/master/src/solaris/classes/sun/awt/X11/XFocusProxyWindow.java#L35
                    if &wm_class != "Focus-Proxy-Window.FocusProxy" {
                        return Some(wm_class);
                    }
                }

                window = get_parent_window(conn, window)?;
            }
        }
        return None;
    }
}

fn get_focus_window(conn: &RustConnection) -> Option<Window> {
    if let Ok(cookie) = xproto::get_input_focus(conn) {
        if let Ok(reply) = cookie.reply() {
            return Some(reply.focus);
        }
    }
    return None;
}

fn get_parent_window(conn: &RustConnection, window: Window) -> Option<Window> {
    if let Ok(cookie) = xproto::query_tree(conn, window) {
        if let Ok(reply) = cookie.reply() {
            return Some(reply.parent);
        }
    }
    return None;
}

fn get_wm_class(conn: &RustConnection, window: Window) -> Option<String> {
    if let Ok(cookie) = get_property(conn, false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024) {
        if let Ok(reply) = cookie.reply() {
            if reply.value.is_empty() {
                return None;
            }

            if let Some(delimiter) = reply.value.iter().position(|byte| *byte == '\0' as u8) {
                if let Ok(prefix) = String::from_utf8(reply.value[..delimiter].to_vec()) {
                    let name = reply.value[(delimiter + 1)..].to_vec();
                    if let Some(end) = name.iter().position(|byte| *byte == '\0' as u8) {
                        if end == name.len() - 1 {
                            if let Ok(name) = String::from_utf8(name[..end].to_vec()) {
                                return Some(format!("{prefix}.{name}"));
                            }
                        }
                    }
                }
            }
        }
    }
    return None;
}
