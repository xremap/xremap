use crate::client::Client;

pub struct NullClient {
    called: bool,
}

impl NullClient {
    pub fn new() -> NullClient {
        NullClient { called: false }
    }
}

impl Client for NullClient {
    fn current_wm_class(&mut self) -> Option<String> {
        if !self.called {
            self.called = true;
            println!("NullClient.supported = false");
        }
        None
    }
}
