use crate::client::Client;
use anyhow::{bail, Result};
use std::env;
use x11rb::connection::Connection;
use x11rb::cookie::Cookie;
use x11rb::protocol::xproto::{self};
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::ConnectionError;
use x11rb::x11_utils::TryParse;
use x11rb::{protocol::xproto::get_property, rust_connection::RustConnection};

pub struct X11Client {
    connection: Option<RustConnection>,
    screen_num: Option<usize>,
}

impl X11Client {
    pub fn new() -> X11Client {
        X11Client {
            connection: None,
            screen_num: None,
        }
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
            Ok((connection, screen)) => {
                self.connection = Some(connection);
                self.screen_num = Some(screen);
            }
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
    fn current_window(&mut self) -> Option<String> {
        self.connect();

        match get_focused_title(self) {
            Ok(x) => Some(x),
            Err(e) => {
                println!("Error when fetching window title: {e:?}");
                // Drop connection so it might work next time.
                self.connection = None;
                None
            }
        }
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

            if window == 0 {
                // No more parents, so fall back to using _NET_ACTIVE_WINDOW
                return current_application_fallback(self);
            }
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
            if let Some(end) = name.iter().position(|byte| *byte == '\0' as u8).or(Some(name.len())) {
                if end == name.len() - 1 || end == name.len() {
                    if let Ok(name) = String::from_utf8(name[..end].to_vec()) {
                        return Some(format!("{prefix}.{name}"));
                    }
                }
            }
        }
    }
    return None;
}

/// Get WM_CLASS by using _NET_ACTIVE_WINDOW
fn current_application_fallback(client: &mut X11Client) -> Option<String> {
    let winid = get_focused_window_id(client.connection.as_ref()?, client.screen_num?).ok()?;

    get_wm_class(client, winid)
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

fn get_focused_title(client: &X11Client) -> Result<String> {
    let conn = client
        .connection
        .as_ref()
        .ok_or_else(|| anyhow::format_err!("Should already be connected"))?;

    let screen_num = client
        .screen_num
        .ok_or_else(|| anyhow::format_err!("Screen_num should be available"))?;

    let winid = get_focused_window_id(conn, screen_num)?;

    let atoms = Atoms::new(&conn)?.reply()?;

    // Get title
    let prop_reply = conn
        .get_property(false, winid, atoms._NET_WM_NAME, atoms.UTF8_STRING, 0, u32::MAX)?
        .reply()?;

    if prop_reply.type_ != x11rb::NONE {
        return Ok(String::from_utf8(prop_reply.value)?);
    }

    // Fallback
    let prop_reply = conn
        .get_property(false, winid, AtomEnum::WM_NAME, AtomEnum::STRING, 0, u32::MAX)?
        .reply()?;

    Ok(String::from_utf8(prop_reply.value)?)
}

/// This is a better alternative to the existing function: get_focus_window
/// Because xproto::get_input_focus is not a reliable way to get focused window.
fn get_focused_window_id(conn: &RustConnection, screen_num: usize) -> anyhow::Result<u32> {
    let root = conn.setup().roots[screen_num].root;

    let atoms = Atoms::new(&conn)?.reply()?;

    // Get _NET_ACTIVE_WINDOW
    let prop_reply = conn
        .get_property(false, root, atoms._NET_ACTIVE_WINDOW, AtomEnum::WINDOW, 0, 1)?
        .reply()?;

    if prop_reply.type_ != x11rb::NONE && prop_reply.value.len() == 4 {
        let arr: [u8; 4] = prop_reply.value.try_into().expect("Should be vector of 4 bytes");

        let winid = u32::from_le_bytes(arr);

        if winid != 0 {
            return Ok(winid);
        }
    }

    // Fallback to focused element which sometimes is the window
    // but could also be an UI element within the active window.
    Ok(conn.get_input_focus()?.reply()?.focus)
}

x11rb::atom_manager! {
    pub Atoms: AtomsCookie {
        _NET_WM_NAME,
        _NET_ACTIVE_WINDOW,
        UTF8_STRING,
    }
}
