use crate::config::deserialize_single_field;
use crate::config::deserializers::{deserialize_duration, DurationWrapper, VectorOrSingleOrNull};
use crate::config::key::deserialize_key;
use crate::config::modmap::KeyWrapper;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpmapOperator {
    DoubleTap(DoubleTap),
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
pub struct DoubleTap {
    #[serde(rename = "double", deserialize_with = "deserialize_expmap_actions")]
    pub actions: Vec<ExpmapAction>,
    #[serde(default = "default_dbltap_timeout", deserialize_with = "deserialize_duration")]
    pub timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ExpmapAction {
    #[serde(deserialize_with = "deserialize_key")]
    Key(Key),
}

pub fn deserialize_expmap_actions<'de, D>(deserializer: D) -> Result<Vec<ExpmapAction>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(VectorOrSingleOrNull::deserialize(deserializer)?.into_vec())
}

fn default_dbltap_timeout() -> Duration {
    Duration::from_millis(200)
}
