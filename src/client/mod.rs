pub trait Client {
    fn supported(&mut self) -> bool;
    fn current_application(&mut self) -> Option<String>;
    fn current_window(&mut self) -> Option<String>;
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
    pub fn current_window(&mut self) -> Option<String> {
        if self.supported.is_none() {
            let supported = self.client.supported();
            self.supported = Some(supported);
            println!("application-client: {} (supported: {})", self.name, supported);
        }
        if !self.supported.unwrap() {
            return None;
        }

        let result = self.client.current_window();
        if let Some(window) = &result {
            if &self.last_window != window {
                self.last_window = window.clone();
                println!("window: {}", window);
            }
        }
        result
    }

    pub fn current_application(&mut self) -> Option<String> {
        if self.supported.is_none() {
            let supported = self.client.supported();
            self.supported = Some(supported);
            println!("application-client: {} (supported: {})", self.name, supported);
        }
        if !self.supported.unwrap() {
            return None;
        }

        let result = self.client.current_application();
        if let Some(application) = &result {
            if &self.last_application != application {
                self.last_application = application.clone();
                println!("application: {}", application);
            }
        }
        result
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

#[cfg(feature = "sway")]
mod sway_client;
#[cfg(feature = "sway")]
pub fn build_client() -> WMClient {
    WMClient::new("Sway", Box::new(sway_client::SwayClient::new()))
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

#[cfg(not(any(
    feature = "gnome",
    feature = "sway",
    feature = "x11",
    feature = "hypr",
    feature = "kde"
)))]
mod null_client;
#[cfg(not(any(
    feature = "gnome",
    feature = "sway",
    feature = "x11",
    feature = "hypr",
    feature = "kde",
    feature = "wlroots"
)))]
pub fn build_client() -> WMClient {
    WMClient::new("none", Box::new(null_client::NullClient))
}
