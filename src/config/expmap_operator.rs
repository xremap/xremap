use crate::config::deserialize_single_field;
use crate::config::deserializers::DurationWrapper;
use crate::config::expmap_actions::{ExpmapActions, ParseExpmapActions};
use crate::config::modmap::KeyWrapper;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpmapOperator {
    DoubleTap(ParseDoubleTap),
    #[serde(deserialize_with = "deserialize_throttle")]
    Throttle(Duration),
    #[serde(deserialize_with = "deserialize_oneshot")]
    OneShot(Key),
    #[serde(deserialize_with = "deserialize_select")]
    Select(Vec<ExpmapOperator>),
}

pub fn deserialize_throttle<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    Ok(deserialize_single_field::<D, DurationWrapper>(deserializer, "throttle_ms")?.0)
}

pub fn deserialize_oneshot<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Key, D::Error> {
    Ok(deserialize_single_field::<D, KeyWrapper>(deserializer, "oneshot")?.0)
}

pub fn deserialize_select<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<ExpmapOperator>, D::Error> {
    Ok(deserialize_single_field::<D, Vec<ExpmapOperator>>(deserializer, "select")?)
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParseDoubleTap {
    pub double: ParseExpmapActions,
    pub timeout: Option<u64>,
}

impl ParseDoubleTap {
    pub fn finish(self) -> DoubleTapDefinition {
        DoubleTapDefinition {
            actions: self.double.finish(),
            timeout: Duration::from_millis(self.timeout.unwrap_or(200)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DoubleTapDefinition {
    pub actions: ExpmapActions,
    pub timeout: Duration,
}
