use crate::client::{build_client, WMClient};

pub struct MainController {
    wmclient: WMClient,
}

impl MainController {
    pub fn new(log_window_changes: bool) -> Self {
        Self {
            wmclient: build_client(log_window_changes),
        }
    }

    pub fn wmclient<'a>(&'a mut self) -> &'a mut WMClient {
        &mut self.wmclient
    }
}
