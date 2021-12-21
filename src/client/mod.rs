#[cfg(feature = "sway")]
mod sway_client;

#[cfg(feature = "x11")]
mod x11_client;

#[cfg(not(any(feature = "sway", feature = "x11")))]
mod null_client;

trait Client {
    fn current_wm_class(&mut self) -> Option<String>;
}

pub struct WMClient {
    client: Box<dyn Client>,
}

impl WMClient {
    pub fn current_wm_class(&mut self) -> Option<String> {
        self.client.current_wm_class()
    }
}

#[cfg(feature = "sway")]
pub fn build_client() -> WMClient {
    WMClient {
        client: Box::new(sway_client::SwayClient::new()),
    }
}

#[cfg(feature = "x11")]
pub fn build_client() -> WMClient {
    WMClient {
        client: Box::new(x11_client::X11Client::new()),
    }
}

#[cfg(not(any(feature = "sway", feature = "x11")))]
pub fn build_client() -> WMClient {
    WMClient {
        client: Box::new(null_client::NullClient::new()),
    }
}
