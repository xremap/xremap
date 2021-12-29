use crate::config::action::Action;
use crate::config::application::Application;
use crate::config::key_press::KeyPress;
use serde::de::{value, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keymap {
    #[serde(default = "String::new")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_remap")]
    pub remap: HashMap<KeyPress, Vec<Action>>,
    pub application: Option<Application>,
}

pub fn deserialize_remap<'de, D>(deserializer: D) -> Result<HashMap<KeyPress, Vec<Action>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct RemapVisitor;

    impl<'de> Visitor<'de> for RemapVisitor {
        type Value = HashMap<KeyPress, Vec<Action>>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("map from string to strings or maps")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut keymap = HashMap::new();

            while let Some(key_press) = map.next_key::<KeyPress>()? {
                let actions = match map.next_value::<Actions>()? {
                    Actions::Action(action) => vec![action],
                    Actions::Actions(actions) => actions,
                };
                keymap.insert(key_press, actions);
            }

            Ok(keymap)
        }
    }

    deserializer.deserialize_any(RemapVisitor)
}

enum Actions {
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
                E: serde::de::Error,
            {
                let key_press = Deserialize::deserialize(value.into_deserializer())?;
                Ok(Actions::Action(Action::KeyPress(key_press)))
            }

            fn visit_seq<S>(self, seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess<'de>,
            {
                let actions = Deserialize::deserialize(value::SeqAccessDeserializer::new(seq))?;
                Ok(Actions::Actions(actions))
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let action = Deserialize::deserialize(value::MapAccessDeserializer::new(map))?;
                Ok(Actions::Action(action))
            }
        }

        deserializer.deserialize_any(ActionsVisitor)
    }
}
