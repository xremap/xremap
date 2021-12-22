#[cfg(feature = "sway")]
mod sway_client;

#[cfg(feature = "x11")]
mod x11_client;

#[cfg(not(any(feature = "sway", feature = "x11")))]
mod null_client;

trait Client {
    fn supported(&mut self) -> bool;
    fn current_application(&mut self) -> Option<String>;
}

pub struct WMClient {
    name: String,
    client: Box<dyn Client>,
    called: bool,
    last_application: String,
}

impl WMClient {
    fn new(name: &str, client: Box<dyn Client>) -> WMClient {
        WMClient {
            name: name.to_string(),
            client,
            called: false,
            last_application: String::new(),
        }
    }

    pub fn current_application(&mut self) -> Option<String> {
        if !self.called {
            self.called = true;
            println!("application-client: {} (supported: {})", self.name, self.client.supported());
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

#[cfg(feature = "sway")]
pub fn build_client() -> WMClient {
    WMClient::new("Sway", Box::new(sway_client::SwayClient::new()))
}

#[cfg(feature = "x11")]
pub fn build_client() -> WMClient {
    WMClient::new("X11", Box::new(x11_client::X11Client::new()))
}

#[cfg(not(any(feature = "sway", feature = "x11")))]
pub fn build_client() -> WMClient {
    WMClient::new("none", Box::new(null_client::NullClient))
}
