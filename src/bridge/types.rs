use crate::client::WindowInfo;
use serde::{Deserialize, Serialize};

// Used by gnome extension. But not the bridge.
#[cfg(any(test, feature = "gnome", feature = "socket"))]
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ActiveWindow {
    #[serde(default)]
    pub wm_class: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Request {
    ActiveWindow,
    Run(Vec<String>),
    WindowList,
    CloseByAppClass(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Response {
    Ok,
    Error(String),
    ActiveWindow { title: String, wm_class: String },
    WindowList(Vec<WindowInfo>),
}

#[test]
fn test_bridge_request() {
    assert_eq!(Request::ActiveWindow, serde_json::from_str::<Request>("\"ActiveWindow\"").unwrap());
    assert_eq!(Request::Run(vec!["foo".into()]), serde_json::from_str::<Request>("{\"Run\": [\"foo\"]}").unwrap());
    assert_eq!(Request::WindowList, serde_json::from_str::<Request>("\"WindowList\"").unwrap());
}

#[test]
fn test_bridge_response() {
    assert_eq!(Response::Ok, serde_json::from_str::<Response>("\"Ok\"").unwrap());
    assert_eq!(
        Response::ActiveWindow {
            title: "foo".into(),
            wm_class: "bar".into()
        },
        serde_json::from_str::<Response>("{\"ActiveWindow\":{\"title\":\"foo\",\"wm_class\":\"bar\"}}\n").unwrap()
    );
}

#[test]
fn test_legacy_bridge_response() {
    // Used by gnome extension
    assert_eq!(
        ActiveWindow {
            title: "foo".into(),
            wm_class: "bar".into()
        },
        serde_json::from_str::<ActiveWindow>("{\"title\":\"foo\",\"wm_class\":\"bar\"}\n").unwrap()
    );
}
