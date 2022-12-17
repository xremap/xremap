use crate::client::Client;
use anyhow::bail;
use std::env;
use x11rb::cookie::Cookie;
use x11rb::protocol::xproto::{self};
use x11rb::protocol::xproto::{AtomEnum, Window};
use x11rb::rust_connection::ConnectionError;
use x11rb::x11_utils::TryParse;
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
        self.reconnect();
    }

    fn reconnect(&mut self) {
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
        let mut window = get_focus_window(self)?;
        loop {
            if let Some(wm_class) = get_wm_class(self, window) {
                // Workaround: https://github.com/JetBrains/jdk8u_jdk/blob/master/src/solaris/classes/sun/awt/X11/XFocusProxyWindow.java#L35
                if &wm_class != "Focus-Proxy-Window.FocusProxy" {
                    return Some(wm_class);
                }
            }

            window = get_parent_window(self, window)?;
        }
    }
}

fn get_focus_window(client: &mut X11Client) -> Option<Window> {
    get_cookie_reply_with_reconnect(client, xproto::get_input_focus)
        .map(|reply| reply.focus)
        .ok()
}

fn get_parent_window(client: &mut X11Client, window: Window) -> Option<Window> {
    get_cookie_reply_with_reconnect(client, |conn| xproto::query_tree(conn, window))
        .map(|reply| reply.parent)
        .ok()
}

fn get_wm_class(client: &mut X11Client, window: Window) -> Option<String> {
    let reply = get_cookie_reply_with_reconnect(client, |conn| {
        get_property(conn, false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)
    })
    .ok()?;

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
    return None;
}

fn get_cookie_reply_with_reconnect<T: TryParse>(
    client: &mut X11Client,
    get_cookie: impl Fn(&RustConnection) -> Result<Cookie<RustConnection, T>, ConnectionError>,
) -> anyhow::Result<T> {
    return match get_cookie_reply(client, &get_cookie) {
        Err(e) => {
            println!("Reconnecting to X11 due to error: {}", e);
            client.reconnect();
            get_cookie_reply(client, &get_cookie)
        }
        x => x,
    };
}

fn get_cookie_reply<T: TryParse>(
    client: &X11Client,
    get_cookie: &impl Fn(&RustConnection) -> Result<Cookie<RustConnection, T>, ConnectionError>,
) -> anyhow::Result<T> {
    match &client.connection {
        Some(conn) => return Ok(get_cookie(conn)?.reply()?),
        None => bail!("No connection to X11"),
    }
}
