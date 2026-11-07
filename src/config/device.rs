use crate::config::deserializers::deserialize_string_or_vec;
use crate::device::InputDeviceInfo;
use serde::Deserialize;

// Used for both keymap and modmap device matching.
//
// Matching semantics (same as ApplicationMatch):
// - `only`: matches if ANY of the matchers matches (OR).
// - `not`:  matches if NONE of the matchers matches.
// - Both set: both conditions must hold (AND).
// - Neither set: never matches (mapping only applies with an explicit matcher).
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
            if !device_only.iter().any(|m| device.matches(m)) {
                return false;
            }
        }
        if let Some(device_not) = &self.not {
            if device_not.iter().any(|m| device.matches(m)) {
                return false;
            }
        }
        self.only.is_some() || self.not.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn device_info(name: &str) -> InputDeviceInfo {
        InputDeviceInfo {
            name: name.into(),
            path: PathBuf::from("/dev/input/event0"),
            vendor: 0x1234,
            product: 0x5678,
        }
    }

    fn matches_config(only: Option<&[&str]>, not: Option<&[&str]>, device: &InputDeviceInfo) -> bool {
        DeviceMatcher {
            only: only.map(|v| v.iter().map(|s| s.to_string()).collect()),
            not: not.map(|v| v.iter().map(|s| s.to_string()).collect()),
        }
        .matches(device)
    }

    #[test]
    fn test_match_only() {
        let device = device_info("Cool Device");
        assert!(matches_config(Some(&["Cool Device"]), None, &device));
        assert!(!matches_config(Some(&["Other Device"]), None, &device));
    }

    #[test]
    fn test_match_not() {
        let device = device_info("Cool Device");
        assert!(matches_config(None, Some(&["Other Device"]), &device));
        assert!(!matches_config(None, Some(&["Cool Device"]), &device));
    }

    #[test]
    fn test_match_only_and_not_combined() {
        // only AND not must both hold
        let device = device_info("Cool Device");
        assert!(matches_config(Some(&["Cool"]), Some(&["Other"]), &device));
        // only matches but not also matches: both must hold, so no match
        assert!(!matches_config(Some(&["Cool"]), Some(&["Cool Device"]), &device));
        assert!(!matches_config(Some(&["Other"]), Some(&["Other"]), &device));
    }

    #[test]
    fn test_match_neither_set() {
        assert!(!matches_config(None, None, &device_info("Cool Device")));
    }
}
