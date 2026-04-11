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

/// This should properly be in the test module.
/// But then wouldn't main-module be able to load it.
#[cfg(feature = "device-test")]
pub struct DeviceTestClient;

#[cfg(feature = "device-test")]
impl Client for DeviceTestClient {
    fn supported(&mut self) -> bool {
        true
    }

    fn current_window(&mut self) -> Option<String> {
        None
    }

    fn current_application(&mut self) -> Option<String> {
        None
    }

    fn run(&mut self, command: &Vec<String>) -> anyhow::Result<bool> {
        println!("{:?}", command);
        Ok(true)
    }

    fn window_list(&mut self) -> anyhow::Result<Vec<WindowInfo>> {
        Ok(vec![])
    }
}
