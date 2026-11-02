use crate::config::key_combo::EmitMode;
use crate::config::keymap_action::{KeymapAction, ParseKeymapAction};
use serde::{de, Deserialize, Deserializer};

// timeout_key is the only/first place simple actions are used. So tests lie there.

// A subset of the actions, that keymap has. Actions, mutating matching_state in keymap
//  are denied. Because knownledge of how they should work, when emitted from the
//  elsewhere, is still missing.
#[derive(Debug, Clone)]
pub struct ParseSimpleAction(ParseKeymapAction);

impl ParseSimpleAction {
    pub fn finish(actions: Vec<ParseSimpleAction>, mode: EmitMode) -> Vec<KeymapAction> {
        ParseKeymapAction::finish(actions.into_iter().map(|action| action.0).collect(), mode)
    }
}

impl<'de> Deserialize<'de> for ParseSimpleAction {
    /// Errors must be given here, to ensure right line numbers.
    /// Injection of default must be delayed to `finish()`, because
    /// there's no context available at parsing.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        macro_rules! errmsg {
            ($arg:tt) => {{
                Err(de::Error::custom(format!("{} is not supported in this context", $arg)))
            }};
        }

        match ParseKeymapAction::deserialize(deserializer)? {
            ParseKeymapAction::Remap(_) => errmsg!("Remap"),
            ParseKeymapAction::SetMode(_) => errmsg!("set_mode"),
            ParseKeymapAction::SetMark(_) => errmsg!("set_mark"),
            ParseKeymapAction::WithMark(_) => errmsg!("with_mark"),
            ParseKeymapAction::EscapeNextKey(_) => errmsg!("escape_next_key"),
            data => Ok(ParseSimpleAction(data)),
        }
    }
}
