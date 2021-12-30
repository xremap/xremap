use crate::config::key_press::KeyPress;
use std::collections::HashMap;

use crate::config::actions::Actions;
use serde::de;
use serde::de::{IntoDeserializer, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::{Debug, Formatter};

// Values in `keymap.remap`
#[derive(Clone, Debug)]
pub enum Action {
    KeyPress(KeyPress),
    Remap(HashMap<KeyPress, Vec<Action>>),
}

impl<'de> Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ActionVisitor;

        impl<'de> Visitor<'de> for ActionVisitor {
            type Value = Action;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("string or map")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let key_press = Deserialize::deserialize(value.into_deserializer())?;
                Ok(Action::KeyPress(key_press))
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let key = map.next_key::<String>()?;
                let action = match key.as_deref() {
                    Some("remap") => {
                        let mut action: HashMap<KeyPress, Vec<Action>> = HashMap::new();
                        let remap = map.next_value::<HashMap<KeyPress, Actions>>()?;
                        for (key_press, actions) in remap.into_iter() {
                            let actions = match actions {
                                Actions::Action(action) => vec![action],
                                Actions::Actions(actions) => actions,
                            };
                            action.insert(key_press, actions);
                        }
                        Action::Remap(action)
                    }
                    Some(action) => return serde_error::<Self::Value, M>(&format!("unexpected action '{}'", action)),
                    None => return serde_error::<Self::Value, M>("missing action"),
                };
                if let Some(key) = map.next_key::<String>()? {
                    return serde_error::<Self::Value, M>(&format!(
                        "only one action key is expected but also got: {}",
                        key
                    ));
                }
                Ok(action)
            }
        }

        deserializer.deserialize_any(ActionVisitor)
    }
}

pub fn serde_error<'de, V, M>(message: &str) -> Result<V, M::Error>
where
    M: MapAccess<'de>,
{
    let error: Box<dyn std::error::Error> = message.into();
    Err(error).map_err(de::Error::custom)
}
