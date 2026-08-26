use serde::{Deserialize, Deserializer};
use std::time::Duration;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum VecOrSingle<T> {
    Vec(Vec<T>),
    Single(T),
}

impl<T> VecOrSingle<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            VecOrSingle::Vec(vec) => vec,
            VecOrSingle::Single(string) => vec![string],
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum VectorOrSingleOrNull<T> {
    NoAction,
    Action(T),
    Actions(Vec<T>),
}

impl<T> VectorOrSingleOrNull<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            VectorOrSingleOrNull::NoAction => vec![],
            VectorOrSingleOrNull::Action(action) => vec![action],
            VectorOrSingleOrNull::Actions(actions) => actions,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct DurationWrapper(#[serde(deserialize_with = "deserialize_duration")] pub Duration);

pub fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = u64::deserialize(deserializer)?;
    Ok(Duration::from_millis(millis))
}
