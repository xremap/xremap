use crate::config::application::ApplicationMatch;
use crate::main_impl::Desktop;
use crate::util::print_table;
use log::{debug, error, info, warn};
use nix::libc::getuid;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[cfg(feature = "cosmic")]
mod cosmic_client;
#[cfg(feature = "cosmic")]
mod cosmic_protocols;
#[cfg(feature = "gnome")]
mod gnome_client;
#[cfg(feature = "hypr")]
mod hypr_client;
#[cfg(feature = "kde")]
mod kde;
#[cfg(feature = "niri")]
mod niri_client;
#[cfg(feature = "pantheon")]
mod pantheon_client;
#[cfg(feature = "socket")]
mod socket_client;
#[cfg(feature = "socket")]
mod socket_monitor;
#[cfg(feature = "wlroots")]
mod wlroots_client;
#[cfg(feature = "x11")]
mod x11_client;

pub mod null_client;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WindowInfo {
    // The order of fields matters because they define sort order.
    pub app_class: Option<String>,
    pub title: Option<String>,
    pub winid: Option<String>,
}

pub trait Client {
    /// Must not print/log anything. Must return an error if connection isn't possible.
    /// This function must be called as the first thing, otherwise it's ill-defined.
    /// Because some clients cache the connection and will never retry connecting.
    fn test_connection(&mut self) -> anyhow::Result<()>;
    // It's called very late. I.e. the first time xremap wants some information.
    // Some clients may print/log messages.
    fn supported(&mut self) -> bool;
    fn current_application(&mut self) -> Option<String>;
    fn current_window(&mut self) -> Option<String>;
    fn run(&mut self, _command: &Vec<String>) -> anyhow::Result<bool> {
        // Ok(false) means the client cannot run the command (try another way)
        // Ok(true) means the command was run successfully
        // Err(...) means there was an error running the command
        Ok(false)
    }
    /// Return a list of open windows
    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>>;
    fn close_windows_by_app_class(&mut self, app_class: &str) -> anyhow::Result<()>;
}

pub struct WMClient {
    pub name: String,
    pub client: Box<dyn Client>,
    // Cached value of calling `client.supported()`
    supported: Option<bool>,
    // The last app_class logged to console
    last_application: String,
    // The last title logged to console
    last_window: String,
    // Log app_class and window changes to console.
    log_window_changes: bool,
    // Cache to reduce use of clients.
    application_cache: Option<String>,
    // Cache to reduce use of clients.
    title_cache: Option<String>,
}

impl WMClient {
    pub fn new(name: &str, client: Box<dyn Client>, log_window_changes: bool) -> WMClient {
        WMClient {
            name: name.to_string(),
            client,
            supported: None,
            last_application: String::new(),
            last_window: String::new(),
            log_window_changes,
            application_cache: None,
            title_cache: None,
        }
    }

    fn supported(&mut self) -> bool {
        if self.supported.is_none() {
            let supported = self.client.supported();
            self.supported = Some(supported);
            println!("application-client: {} (supported: {})", self.name, supported);
        }
        self.supported.unwrap()
    }

    pub fn current_window(&mut self) -> Option<String> {
        if !self.supported() {
            return None;
        }

        let result = self.client.current_window();
        if self.log_window_changes {
            if let Some(window) = &result {
                if &self.last_window != window {
                    self.last_window = window.clone();
                    println!("window: {window}");
                }
            }
        }
        result
    }

    pub fn current_application(&mut self) -> Option<String> {
        if !self.supported() {
            return None;
        }

        let result = self.client.current_application();
        if self.log_window_changes {
            if let Some(application) = &result {
                if &self.last_application != application {
                    self.last_application = application.clone();
                    println!("application: {application}");
                }
            }
        }
        result
    }

    pub fn run(&mut self, command: &Vec<String>) -> anyhow::Result<bool> {
        if self.supported() {
            self.client.run(command)
        } else {
            Ok(false)
        }
    }

    pub fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        self.client.window_list()
    }

    pub fn close_windows_by_app_class(&mut self, app_class: &str) -> anyhow::Result<()> {
        self.client.close_windows_by_app_class(app_class)
    }

    pub fn clear_app_class_and_title(&mut self) {
        self.application_cache = None; // expire cache
        self.title_cache = None; // expire cache
    }

    pub fn match_window(&mut self, window_matcher: &ApplicationMatch) -> bool {
        // Lazily fill the wm_class cache
        if self.title_cache.is_none() {
            let title = self.current_window().unwrap_or_default();
            self.title_cache = Some(title);
        }

        if let Some(title) = &self.title_cache {
            window_matcher.matches(title)
        } else {
            false
        }
    }

    pub fn match_application(&mut self, application_matcher: &ApplicationMatch) -> bool {
        // Lazily fill the wm_class cache
        if self.application_cache.is_none() {
            let application = self.current_application().unwrap_or_default();
            self.application_cache = Some(application);
        }

        if let Some(application) = &self.application_cache {
            application_matcher.matches(application)
        } else {
            false
        }
    }
}

fn supported_clients(_log_window_changes: bool) -> Vec<(Desktop, &'static str, Box<dyn Fn() -> Box<dyn Client>>)> {
    // X11 is last. Because it's generic, and one of the others should
    // be used if possible. It must also be last because a Wayland desktop
    // with XWayland-support likely isolates the apps.
    // Niri and Hypr must be before wlroots. They aren't selected, otherwise.
    vec![
        #[cfg(feature = "gnome")]
        (Desktop::Gnome, "GNOME", Box::new(|| Box::new(gnome_client::GnomeClient::new()))),
        #[cfg(feature = "kde")]
        (Desktop::Kde, "KDE", Box::new(move || Box::new(kde::KdeClient::new(_log_window_changes)))),
        #[cfg(feature = "hypr")]
        (Desktop::Hypr, "Hypr", Box::new(|| Box::new(hypr_client::HyprlandClient::new()))),
        #[cfg(feature = "niri")]
        (Desktop::Niri, "Niri", Box::new(|| Box::new(niri_client::NiriClient::new()))),
        #[cfg(feature = "wlroots")]
        (Desktop::Wlroots, "wlroots", Box::new(|| Box::new(wlroots_client::WlRootsClient::new()))),
        #[cfg(feature = "cosmic")]
        (Desktop::Cosmic, "COSMIC", Box::new(|| Box::new(cosmic_client::CosmicClient::new()))),
        #[cfg(feature = "pantheon")]
        (Desktop::Pantheon, "Pantheon", Box::new(|| Box::new(pantheon_client::PantheonClient::new()))),
        #[cfg(feature = "x11")]
        (Desktop::X11, "X11", Box::new(|| Box::new(x11_client::X11Client::new()))),
        #[cfg(feature = "socket")]
        (Desktop::Socket, "Socket", Box::new(|| Box::new(socket_client::SocketClient::new()))),
    ]
}

fn concat_supported_clients(clients: Vec<(Desktop, &'static str, Box<dyn Fn() -> Box<dyn Client>>)>) -> String {
    clients
        .into_iter()
        .map(|(_, name, _)| name)
        .collect::<Vec<_>>()
        .join(", ")
}

pub fn build_client(log_window_changes: bool, desktop: Desktop) -> WMClient {
    #[cfg(feature = "device-test")]
    if 1 == 1 {
        // Runtime guard to allow easy type checking with `--all-features`.
        return WMClient::new("DeviceTest", Box::new(null_client::DeviceTestClient), log_window_changes);
    }

    let clients = supported_clients(log_window_changes);

    match desktop {
        Desktop::None => WMClient::new("none", Box::new(null_client::NullClient), log_window_changes),
        Desktop::Auto => auto_select_client(clients, log_window_changes),
        _ => {
            for (_desktop, name, client) in &clients {
                if desktop == *_desktop {
                    return WMClient::new(name, client(), log_window_changes);
                }
            }

            let supported = concat_supported_clients(clients);
            error!("This variant of xremap doesn't support '{desktop:?}'. Supported: {supported}");
            WMClient::new("none", Box::new(null_client::NullClient), log_window_changes)
        }
    }
}

pub fn auto_select_client(
    clients: Vec<(Desktop, &str, Box<dyn Fn() -> Box<dyn Client>>)>,
    log_window_changes: bool,
) -> WMClient {
    // Desktop chosen at compile time
    if clients.len() == 0 {
        return WMClient::new("none", Box::new(null_client::NullClient), log_window_changes);
    } else if clients.len() == 1 {
        let (_, name, client) = clients.into_iter().next().unwrap();
        return WMClient::new(name, client(), log_window_changes);
    }

    let start_time = Instant::now();
    let mut supported_clients: Vec<(&str, Box<dyn Client>)> = vec![];

    for (_desktop, name, client) in clients {
        if _desktop == Desktop::Socket {
            // Socket can't be automatically selected.
            // It must also not be constructed, because that would start SessionMonitor.
            continue;
        }

        let mut client = client(); // Construct the client.

        let display_name = format!("{}:", name);
        match client.test_connection() {
            Ok(()) => {
                info!("{:<9} supported.", display_name);
                supported_clients.push((name, client));
            }
            Err(err) => {
                debug!("{:<9} {:?}", display_name, err)
            }
        };
    }

    debug!("Client-select-time: {:?}", Instant::now().duration_since(start_time));

    // Take the first supported
    for (name, client) in supported_clients {
        info!("Using client: {:?}", name);
        return WMClient::new(name, client, log_window_changes);
    }

    warn!("No supported desktop clients. Run with 'RUST_LOG=debug' to get more info.");

    if 1000 > unsafe { getuid() } {
        warn!("When running as system user it's difficult to get a connection to the desktop environment.");
    }

    // When there's nothing supported
    WMClient::new("none", Box::new(null_client::NullClient), log_window_changes)
}

pub fn print_supported_desktops() {
    let supported = concat_supported_clients(supported_clients(false));
    println!("This variant of xremap supports: {supported}");
}

pub fn print_open_windows(desktop: Desktop) -> anyhow::Result<()> {
    let mut wmclient = build_client(false, desktop);

    // This must be done to connect.
    if !wmclient.client.supported() {
        eprintln!("Can't connect to '{}'.", wmclient.name);
        return Ok(());
    }

    let windows = wmclient.window_list()?;
    print_windows(windows)
}

pub fn print_windows(mut windows: Vec<WindowInfo>) -> anyhow::Result<()> {
    windows.sort();

    let mut table: Vec<Vec<String>> = vec![];

    table.push(vec!["APP_CLASS".into(), "TITLE".into(), "WIN_ID".into()]);

    for WindowInfo {
        app_class,
        winid,
        title,
    } in windows
    {
        table.push(vec![
            app_class.unwrap_or_default(),
            title.unwrap_or_default(),
            winid.unwrap_or_default(),
        ]);
    }

    print_table(table);

    Ok(())
}
