use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub enum ActionWithoutArgs {
    Exit,
    ReloadConfig,
    PopWindowInfo,
    PrintWindowInfo,
    PrintWindowList,
}

impl<'de> Deserialize<'de> for ActionWithoutArgs {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let action = String::deserialize(deserializer)?.to_lowercase();

        if action == "exit" {
            Ok(ActionWithoutArgs::Exit)
        } else if action == "reload_config" {
            Ok(ActionWithoutArgs::ReloadConfig)
        } else if action == "pop_window_info" {
            Ok(ActionWithoutArgs::PopWindowInfo)
        } else if action == "print_window_info" {
            Ok(ActionWithoutArgs::PrintWindowInfo)
        } else if action == "print_window_list" {
            Ok(ActionWithoutArgs::PrintWindowList)
        } else {
            Err(serde::de::Error::custom("Action {action} not found."))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::tests::assert_invalid_config;
    use indoc::indoc;

    #[test]
    fn test_action_after_exit_fails() {
        assert_invalid_config(
            indoc! {"
                    keymap:
                      - remap:
                          f12:
                            - { action: exit }
                            - A
                    "
            },
            "Actions after exit or reload_config are not allowed.",
        )
    }

    #[test]
    fn test_action_after_config_reload_fails() {
        assert_invalid_config(
            indoc! {"
                    keymap:
                      - remap:
                          f12:
                            - { action: reload_config }
                            - A
                    "
            },
            "Actions after exit or reload_config are not allowed.",
        )
    }
}
