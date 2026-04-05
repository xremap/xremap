use crate::config::application::deserialize_string_or_vec;
use crate::device::InputDeviceInfo;
use serde::Deserialize;

// TODO: Use trait to allow only either `only` or `not`
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceMatcher {
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub only: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub not: Option<Vec<String>>,
}

impl DeviceMatcher {
    pub fn matches(&self, device: &InputDeviceInfo) -> bool {
        if let Some(device_only) = &self.only {
            return device_only.iter().any(|m| device.matches(m));
        }
        if let Some(device_not) = &self.not {
            return device_not.iter().all(|m| !device.matches(m));
        }
        false
    }
}
