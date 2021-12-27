use crate::config::action::serde_error;
use crate::config::key::parse_key;
use evdev::Key;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;
use std::time::Duration;

static DEFAULT_ALONE_TIMEOUT_MILLIS: u64 = 1000;

// Values in `modmap.remap`
#[derive(Clone, Debug)]
pub enum KeyAction {
    Key(Key),
    MultiPurposeKey(MultiPurposeKey),
}

#[derive(Clone, Debug)]
pub struct MultiPurposeKey {
    pub held: Key,
    pub alone: Key,
    pub alone_timeout: Duration,
}

impl<'de> Deserialize<'de> for KeyAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyActionVisitor;

        impl<'de> Visitor<'de> for KeyActionVisitor {
            type Value = KeyAction;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let key = parse_key(value).map_err(serde::de::Error::custom)?;
                Ok(KeyAction::Key(key))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut held: Option<Key> = None;
                let mut alone: Option<Key> = None;
                let mut alone_timeout_millis: u64 = DEFAULT_ALONE_TIMEOUT_MILLIS;

                while let Some(key) = map.next_key::<String>()? {
                    match &key[..] {
                        "held" => {
                            let value: String = map.next_value()?;
                            held = Some(parse_key(&value).map_err(serde::de::Error::custom)?)
                        }
                        "alone" => {
                            let value: String = map.next_value()?;
                            alone = Some(parse_key(&value).map_err(serde::de::Error::custom)?)
                        }
                        "alone_timeout_millis" => alone_timeout_millis = map.next_value()?,
                        key => {
                            return serde_error::<Self::Value, M>(&format!(
                                "held, alone, or alone_timeout_ms is expected, but got: {}",
                                key
                            ))
                        }
                    }
                }

                let held = match held {
                    Some(held) => held,
                    None => {
                        return serde_error::<Self::Value, M>(
                            "held is not specified in a multi-purpose remap of modmap",
                        )
                    }
                };
                let alone = match alone {
                    Some(alone) => alone,
                    None => {
                        return serde_error::<Self::Value, M>(
                            "alone is not specified in a multi-purpose remap of modmap",
                        )
                    }
                };
                let multi_purpose_key = MultiPurposeKey {
                    held,
                    alone,
                    alone_timeout: Duration::from_millis(alone_timeout_millis),
                };
                Ok(KeyAction::MultiPurposeKey(multi_purpose_key))
            }
        }

        deserializer.deserialize_any(KeyActionVisitor)
    }
}
