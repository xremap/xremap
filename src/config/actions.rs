use crate::config::action::Action;
use crate::config::key_press::parse_key_press;
use serde::de;
use serde::de::{value, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

// Used only for deserializing Vec<Action>
pub enum Actions {
    Action(Action),
    Actions(Vec<Action>),
}

impl<'de> Deserialize<'de> for Actions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ActionsVisitor;

        impl<'de> Visitor<'de> for ActionsVisitor {
            type Value = Actions;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("strings or maps")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let key_press = parse_key_press(value).map_err(de::Error::custom)?;
                Ok(Actions::Action(Action::KeyPress(key_press)))
            }

            fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let actions: Vec<Action> =
                    Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))?;
                Ok(Actions::Actions(actions))
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let action: Action =
                    Deserialize::deserialize(value::MapAccessDeserializer::new(map))?;
                Ok(Actions::Action(action))
            }
        }

        deserializer.deserialize_any(ActionsVisitor)
    }
}
