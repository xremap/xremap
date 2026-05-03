use crate::config::application::OnlyOrNot;
use crate::util::print_table;
use anyhow::Context;
use serde::{Deserialize, Serialize};

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
    // It's called very late. I.e. the first time xremap wants some information.
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

    fn check_supported(&mut self) -> Option<()> {
        if self.supported.is_none() {
            let supported = self.client.supported();
            self.supported = Some(supported);
            println!("application-client: {} (supported: {})", self.name, supported);
        }
        self.supported.unwrap().then_some(())
    }

    pub fn current_window(&mut self) -> Option<String> {
        self.check_supported()?;

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
        self.check_supported()?;

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
        if self.check_supported().is_some() {
            return self.client.run(command);
        }
        Ok(false)
    }

    pub fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        self.client.window_list()
    }

    pub fn close_windows_by_app_class(&mut self, app_class: &str) -> anyhow::Result<()> {
        self.client
            .close_windows_by_app_class(&app_class)
            .context("Failed to close by app_class.")
    }

    pub fn clear_app_class_and_title(&mut self) {
        self.application_cache = None; // expire cache
        self.title_cache = None; // expire cache
    }

    pub fn match_window(&mut self, window_matcher: &OnlyOrNot) -> bool {
        // Lazily fill the wm_class cache
        if self.title_cache.is_none() {
            match self.current_window() {
                Some(title) => self.title_cache = Some(title),
                None => self.title_cache = Some(String::new()),
            }
        }

        if let Some(title) = &self.title_cache {
            if let Some(title_only) = &window_matcher.only {
                return title_only.iter().any(|m| m.matches(title));
            }
            if let Some(title_not) = &window_matcher.not {
                return title_not.iter().all(|m| !m.matches(title));
            }
        }
        false
    }

    pub fn match_application(&mut self, application_matcher: &OnlyOrNot) -> bool {
        // Lazily fill the wm_class cache
        if self.application_cache.is_none() {
            match self.current_application() {
                Some(application) => self.application_cache = Some(application),
                None => self.application_cache = Some(String::new()),
            }
        }

        if let Some(application) = &self.application_cache {
            if let Some(application_only) = &application_matcher.only {
                return application_only.iter().any(|m| m.matches(application));
            }
            if let Some(application_not) = &application_matcher.not {
                return application_not.iter().all(|m| !m.matches(application));
            }
        }
        false
    }
}

pub fn build_client(log_window_changes: bool) -> WMClient {
    let clients: Vec<WMClient> = vec![
        #[cfg(feature = "gnome")]
        WMClient::new("GNOME", Box::new(gnome_client::GnomeClient::new()), log_window_changes),
        #[cfg(feature = "kde")]
        WMClient::new("KDE", Box::new(kde::KdeClient::new(log_window_changes)), log_window_changes),
        #[cfg(feature = "hypr")]
        WMClient::new("Hypr", Box::new(hypr_client::HyprlandClient::new()), log_window_changes),
        #[cfg(feature = "x11")]
        WMClient::new("X11", Box::new(x11_client::X11Client::new()), log_window_changes),
        #[cfg(feature = "wlroots")]
        WMClient::new("wlroots", Box::new(wlroots_client::WlRootsClient::new()), log_window_changes),
        #[cfg(feature = "niri")]
        WMClient::new("Niri", Box::new(niri_client::NiriClient::new()), log_window_changes),
        #[cfg(feature = "cosmic")]
        WMClient::new("COSMIC", Box::new(cosmic_client::CosmicClient::new()), log_window_changes),
        #[cfg(feature = "pantheon")]
        WMClient::new("Pantheon", Box::new(pantheon_client::PantheonClient::new()), log_window_changes),
        #[cfg(feature = "socket")]
        WMClient::new("Socket", Box::new(socket_client::SocketClient::new()), log_window_changes),
        #[cfg(feature = "device-test")]
        WMClient::new("DeviceTest", Box::new(null_client::DeviceTestClient), log_window_changes),
    ];

    if clients.len() == 0 {
        WMClient::new("none", Box::new(null_client::NullClient), log_window_changes)
    } else if clients.len() == 1 {
        clients.into_iter().next().unwrap()
    } else {
        // Shouldn't use panic, but this cannot happen for users,
        // because two features would previously conflict already at
        // compile-time, with multiple declarations of `build_client`.
        panic!("There is no way to run with multiple clients enabled.")
    }
}

pub fn print_open_windows() -> anyhow::Result<()> {
    let mut wmclient = build_client(false);

    // This must be done to connect.
    if !wmclient.client.supported() {
        eprintln!("{} is not supported.", wmclient.name);
        return Ok(());
    }

    let mut windows = wmclient.window_list()?;

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
