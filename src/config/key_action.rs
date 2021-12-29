use crate::config::key::Key;
use serde::de::{IntoDeserializer, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

// Values in `modmap.remap`
#[derive(Clone, Debug)]
pub enum KeyAction {
    Key(Key),
    MultiPurposeKey(MultiPurposeKey),
}

#[derive(Clone, Debug, Deserialize)]
pub struct MultiPurposeKey {
    pub held: Key,
    pub alone: Key,
    #[serde(default = "MultiPurposeKey::default_alone_timeout_millis")]
    pub alone_timeout_millis: u64,
}

impl MultiPurposeKey {
    fn default_alone_timeout_millis() -> u64 {
        1000
    }
}

impl<'de> Deserialize<'de> for KeyAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ActionVisitor;

        impl<'de> Visitor<'de> for ActionVisitor {
            type Value = KeyAction;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let key = Deserialize::deserialize(value.into_deserializer())?;
                Ok(KeyAction::Key(key))
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let multi_purpose_key = Deserialize::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(KeyAction::MultiPurposeKey(multi_purpose_key))
            }
        }

        deserializer.deserialize_any(ActionVisitor)
    }
}
