use evdev::{InputEvent, KeyCode as Key};

pub fn get_pretty_events(value: impl AsRef<Vec<InputEvent>>) -> String {
    let mut result = "".to_string();

    for event in value.as_ref() {
        match event.destructure() {
            evdev::EventSummary::Synchronization(_, _, _) => {
                // ignore
            }
            evdev::EventSummary::Key(event, _, value) => {
                if event.code() == Key::KEY_MOVE {
                    // It's used to sync with xremap process, so not interesting.
                    continue;
                }
                let key = format!("{:?}", event.code()).to_lowercase().replace("key_", "");
                result.push_str(&format!("{key}:{value}\n"));
            }
            _ => {
                todo!()
            }
        }
    }
    result
}
