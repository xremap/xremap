use crate::config::key::deserialize_key;
use evdev::KeyCode as Key;
use serde::{Deserialize, Deserializer};
use serde_with::{serde_as, DurationMilliSeconds};
use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum ExpmapOperator {
    DoubleTap(DoubleTap),
}

#[serde_as]
#[derive(Clone, Debug, Deserialize)]
pub struct DoubleTap {
    #[serde(rename = "double", deserialize_with = "deserialize_expmap_actions")]
    pub actions: Vec<ExpmapAction>,

    #[serde_as(as = "DurationMilliSeconds")]
    #[serde(default = "default_dbltap_timeout")]
    pub timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ExpmapAction {
    #[serde(deserialize_with = "deserialize_key")]
    Key(Key),
}

// Used only for deserializing
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ExpmapActions<T> {
    NoAction,
    Action(T),
    Actions(Vec<T>),
}

impl<T> ExpmapActions<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            ExpmapActions::NoAction => vec![],
            ExpmapActions::Action(action) => vec![action],
            ExpmapActions::Actions(actions) => actions,
        }
    }
}

pub fn deserialize_expmap_actions<'de, D>(deserializer: D) -> Result<Vec<ExpmapAction>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(ExpmapActions::deserialize(deserializer)?.into_vec())
}

fn default_dbltap_timeout() -> Duration {
    Duration::from_millis(200)
}
