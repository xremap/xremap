#![cfg(feature = "device-test")]

use crate::common::key_press;
use crate::common::xremap_controller::XremapController;
use evdev::KeyCode;
use indoc::indoc;
use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::os::unix::net::UnixStream;

mod common;

const ABOVE_MAX_CONNECTIONS: u8 = 100; // Max is 64

#[test]
pub fn e2e_ipc_sets_matching_mode() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config_with_ipc(&indoc! {"
            keymap:
                - mode: new_mode
                  remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    assert_eq!("default\n", ctrl.ipc_client(vec!["get-mode"])?);
    assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", "new_mode"])?);

    // Ipc client changed mode, so now exit is possible.
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;
    ctrl.wait_for_output()?;
    Ok(())
}

#[test]
pub fn e2e_ipc_serial_connections() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config_with_ipc(&indoc! {"
            keymap:
                - mode: new_mode
                  remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    // No problem as long they are closed serially.
    for _ in 0..ABOVE_MAX_CONNECTIONS {
        assert_eq!("default\n", ctrl.ipc_client(vec!["get-mode"])?);
    }

    // Now also change mode
    for i in 0..ABOVE_MAX_CONNECTIONS {
        let mode = format!("new_mode_{i}");
        assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", &mode])?);
        assert_eq!(format!("{mode}\n"), ctrl.ipc_client(vec!["get-mode"])?);
    }

    // Check key handling still works
    assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", "new_mode"])?);
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;
    ctrl.wait_for_output()?;
    Ok(())
}

#[test]
pub fn e2e_ipc_max_connections() -> anyhow::Result<()> {
    let mut ctrl = XremapController::builder()
        .config_with_ipc(&indoc! {"
            keymap:
                - mode: new_mode
                  remap:
                    f12: { action: exit }
            ",
        })?
        .build()?;

    // Expect all requests to succeed until they start all to fail.
    let mut max_reached = false;
    let mut streams = vec![];
    for _ in 0..ABOVE_MAX_CONNECTIONS {
        let reader = UnixStream::connect(ctrl.get_ipc_path()).and_then(|mut stream| {
            stream.write_all(b"get-mode\n")?;
            let mut reader = BufReader::new(stream);
            let mut reply = String::new();
            reader.read_line(&mut reply)?;
            if reply.is_empty() {
                return Err(Error::new(ErrorKind::Other, "xremap closed the connection before replying"));
            }
            Ok(reader)
        });
        match reader {
            Ok(_) => assert!(!max_reached),
            Err(_) => max_reached = true,
        };
        // Keep connections open. Alternative could be `--keep-alive`
        // argument to `xremap-cli get-mode`
        streams.push(reader);
    }

    assert!(max_reached);

    // Free up connections to xremap, so functionality is restored
    drop(streams);
    assert_eq!("default\n", ctrl.ipc_client(vec!["get-mode"])?);

    // Check key handling still works
    assert_eq!("ok\n", ctrl.ipc_client(vec!["set-mode", "new_mode"])?);
    ctrl.emit_events(&vec![key_press(KeyCode::KEY_F12)])?;
    ctrl.wait_for_output()?;
    Ok(())
}
