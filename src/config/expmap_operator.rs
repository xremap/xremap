use crate::config::deserialize_single_field;
use crate::config::expmap_actions::{ExpmapActions, ParseExpmapActions};
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpmapOperator {
    DoubleTap(ParseDoubleTap),
    Throttle(ParseThrottle),
    #[serde(deserialize_with = "deserialize_oneshot")]
    OneShot(ParseExpmapActions),
    #[serde(deserialize_with = "deserialize_select")]
    Select(Vec<ExpmapOperator>),
}

pub fn deserialize_oneshot<'de, D: Deserializer<'de>>(deserializer: D) -> Result<ParseExpmapActions, D::Error> {
    Ok(deserialize_single_field::<D, ParseExpmapActions>(deserializer, "oneshot")?)
}

pub fn deserialize_select<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<ExpmapOperator>, D::Error> {
    Ok(deserialize_single_field::<D, Vec<ExpmapOperator>>(deserializer, "select")?)
}

#[derive(Clone, Debug)]
pub struct DoubleTapDefinition {
    pub actions: ExpmapActions,
    pub timeout: Duration,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ParseDoubleTap {
    pub double: ParseExpmapActions,
    pub timeout: Option<u64>,
}

impl ParseDoubleTap {
    pub fn finish(self, bypass: bool) -> DoubleTapDefinition {
        DoubleTapDefinition {
            actions: self.double.finish(bypass),
            timeout: Duration::from_millis(self.timeout.unwrap_or(200)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThrottleDefinition {
    pub timeout: Duration,
    pub bypass: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParseThrottle {
    pub throttle_ms: u64,
}

impl ParseThrottle {
    pub fn finish(self, bypass: bool) -> ThrottleDefinition {
        ThrottleDefinition {
            timeout: Duration::from_millis(self.throttle_ms),
            bypass,
        }
    }
}
