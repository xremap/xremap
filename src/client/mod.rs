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
#[cfg(feature = "wlroots")]
mod wlroots_client;
#[cfg(feature = "x11")]
mod x11_client;

mod null_client;

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
                println!("window: {window}");
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
                println!("application: {application}");
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

pub fn build_client() -> WMClient {
    let clients: Vec<WMClient> = vec![
        #[cfg(feature = "gnome")]
        WMClient::new("GNOME", Box::new(gnome_client::GnomeClient::new())),
        #[cfg(feature = "kde")]
        WMClient::new("KDE", Box::new(kde_client::KdeClient::new())),
        #[cfg(feature = "hypr")]
        WMClient::new("Hypr", Box::new(hypr_client::HyprlandClient::new())),
        #[cfg(feature = "x11")]
        WMClient::new("X11", Box::new(x11_client::X11Client::new())),
        #[cfg(feature = "wlroots")]
        WMClient::new("wlroots", Box::new(wlroots_client::WlRootsClient::new())),
        #[cfg(feature = "niri")]
        WMClient::new("Niri", Box::new(niri_client::NiriClient::new())),
        #[cfg(feature = "cosmic")]
        WMClient::new("COSMIC", Box::new(cosmic_client::CosmicClient::new())),
    ];

    if clients.len() == 0 {
        WMClient::new("none", Box::new(null_client::NullClient))
    } else if clients.len() == 1 {
        clients.into_iter().next().unwrap()
    } else {
        // Shouldn't use panic, but this cannot happen for users,
        // because two features would previously conflict already at
        // compile-time, with multiple declarations of `build_client`.
        panic!("There is no way to run with multiple clients enabled.")
    }
}
