use crate::util::print_table;

#[cfg(feature = "cosmic")]
mod cosmic_client;
#[cfg(feature = "cosmic")]
mod cosmic_protocols;
#[cfg(feature = "gnome")]
mod gnome_client;
#[cfg(feature = "hypr")]
mod hypr_client;
#[cfg(feature = "kde")]
mod kde_client;
#[cfg(feature = "niri")]
mod niri_client;
#[cfg(feature = "socket")]
mod socket_client;
#[cfg(feature = "socket")]
mod socket_monitor;
#[cfg(feature = "wlroots")]
mod wlroots_client;
#[cfg(feature = "x11")]
mod x11_client;

mod null_client;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowInfo {
    // The order of fields matters because they define sort order.
    pub app_class: Option<String>,
    pub title: Option<String>,
    pub win_id: Option<String>,
}

pub trait Client {
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
}

pub struct WMClient {
    name: String,
    client: Box<dyn Client>,
    supported: Option<bool>,
    last_application: String,
    last_window: String,
    log_window_changes: bool,
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
        if let Some(window) = &result {
            if &self.last_window != window {
                self.last_window = window.clone();
                if self.log_window_changes {
                    println!("window: {window}");
                }
            }
        }
        result
    }

    pub fn current_application(&mut self) -> Option<String> {
        self.check_supported()?;

        let result = self.client.current_application();
        if let Some(application) = &result {
            if &self.last_application != application {
                self.last_application = application.clone();
                if self.log_window_changes {
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
}

pub fn build_client(log_window_changes: bool) -> WMClient {
    let clients: Vec<WMClient> = vec![
        #[cfg(feature = "gnome")]
        WMClient::new("GNOME", Box::new(gnome_client::GnomeClient::new()), log_window_changes),
        #[cfg(feature = "kde")]
        WMClient::new("KDE", Box::new(kde_client::KdeClient::new(log_window_changes)), log_window_changes),
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
        #[cfg(feature = "socket")]
        WMClient::new("Socket", Box::new(socket_client::SocketClient::new()), log_window_changes),
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
        win_id,
        title,
    } in windows
    {
        table.push(vec![
            app_class.unwrap_or_default(),
            title.unwrap_or_default(),
            win_id.unwrap_or_default(),
        ]);
    }

    print_table(table);

    Ok(())
}
