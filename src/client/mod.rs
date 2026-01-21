use log::debug;

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
}

pub struct WMClient {
    name: String,
    client: Box<dyn Client>,
    supported: Option<bool>,
    last_application: String,
    last_window: String,
}

impl WMClient {
    pub fn new(name: &str, client: Box<dyn Client>) -> WMClient {
        WMClient {
            name: name.to_string(),
            client,
            supported: None,
            last_application: String::new(),
            last_window: String::new(),
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
                debug!("window: {window}");
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
                debug!("application: {application}");
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
}

#[cfg(feature = "gnome")]
mod gnome_client;
#[cfg(feature = "gnome")]
pub fn build_client() -> WMClient {
    WMClient::new("GNOME", Box::new(gnome_client::GnomeClient::new()))
}

#[cfg(feature = "kde")]
mod kde_client;
#[cfg(feature = "kde")]
pub fn build_client() -> WMClient {
    WMClient::new("KDE", Box::new(kde_client::KdeClient::new()))
}

#[cfg(feature = "hypr")]
mod hypr_client;
#[cfg(feature = "hypr")]
pub fn build_client() -> WMClient {
    WMClient::new("Hypr", Box::new(hypr_client::HyprlandClient::new()))
}

#[cfg(feature = "x11")]
mod x11_client;
#[cfg(feature = "x11")]
pub fn build_client() -> WMClient {
    WMClient::new("X11", Box::new(x11_client::X11Client::new()))
}

#[cfg(feature = "wlroots")]
mod wlroots_client;
#[cfg(feature = "wlroots")]
pub fn build_client() -> WMClient {
    WMClient::new("wlroots", Box::new(wlroots_client::WlRootsClient::new()))
}

#[cfg(feature = "niri")]
mod niri_client;
#[cfg(feature = "niri")]
pub fn build_client() -> WMClient {
    WMClient::new("Niri", Box::new(niri_client::NiriClient::new()))
}

#[cfg(feature = "cosmic")]
mod cosmic_client;
#[cfg(feature = "cosmic")]
mod cosmic_protocols;
#[cfg(feature = "cosmic")]
pub fn build_client() -> WMClient {
    WMClient::new("Cosmic", Box::new(cosmic_client::CosmicClient::new()))
}

#[cfg(feature = "socket")]
mod socket_client;
#[cfg(feature = "socket")]
mod socket_monitor;
#[cfg(feature = "socket")]
pub fn build_client() -> WMClient {
    WMClient::new("Socket", Box::new(socket_client::SocketClient::new()))
}

#[cfg(not(any(
    feature = "gnome",
    feature = "x11",
    feature = "hypr",
    feature = "kde",
    feature = "wlroots",
    feature = "niri",
    feature = "cosmic",
    feature = "socket"
)))]
mod null_client;
#[cfg(not(any(
    feature = "gnome",
    feature = "x11",
    feature = "hypr",
    feature = "kde",
    feature = "wlroots",
    feature = "niri",
    feature = "cosmic",
    feature = "socket"
)))]
pub fn build_client() -> WMClient {
    WMClient::new("none", Box::new(null_client::NullClient))
}
