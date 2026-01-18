use crate::client::{Client, WindowInfo};

pub struct NullClient;

impl Client for NullClient {
    fn supported(&mut self) -> bool {
        false
    }
    fn current_window(&mut self) -> Option<String> {
        None
    }

    fn current_application(&mut self) -> Option<String> {
        None
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        Ok(vec![])
    }
}
