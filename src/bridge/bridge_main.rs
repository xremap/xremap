use crate::bridge::{Request, Response};
use crate::client::{build_client, WMClient};
use crate::command_runner::CommandRunner;
use anyhow::{bail, Context};
use nix::unistd::getuid;
use std::fs::{exists, remove_file, set_permissions};
use std::io::{prelude::*, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

pub fn main(log_window_changes: bool, allow_launch: bool) -> anyhow::Result<()> {
    let uid = getuid().as_raw();
    let socket_path = format!("/run/xremap/{uid}/xremap.sock");
    let mut command_runner = CommandRunner::new(allow_launch);
    let mut wmclient = build_client(log_window_changes);

    // This must be done to connect.
    if !wmclient.client.supported() {
        eprintln!("{} is not supported.", wmclient.name);
        return Ok(());
    }

    let listener = get_listener(&socket_path)?;

    for stream in listener.incoming() {
        handle_connection(stream?, &mut wmclient, &mut command_runner)?
    }

    unreachable!()
}

fn get_listener(socket_path: &str) -> anyhow::Result<UnixListener> {
    // Checks
    let dir = Path::new(socket_path)
        .parent()
        .ok_or_else(|| anyhow::format_err!("The socket address must have a parent folder."))?;

    if !exists(dir)? {
        bail!("Parent folder of the socket address must exist: {dir:?}")
    }

    if exists(socket_path)? {
        // TODO: Also clean up on shutdown. Even though it's not reliable.
        remove_file(socket_path)?;
    }

    // Connect
    let listener = UnixListener::bind(socket_path).context("Could not create socket")?;

    set_permissions(socket_path, std::fs::Permissions::from_mode(0o660))
        .context(format!("Can't set permission for socket: {socket_path}"))?;

    Ok(listener)
}

fn handle_connection(
    stream: UnixStream,
    wmclient: &mut WMClient,
    command_runner: &mut CommandRunner,
) -> anyhow::Result<()> {
    // Request
    let mut reader = BufReader::new(stream);
    let mut request = String::new();
    reader.read_line(&mut request)?;

    let request = serde_json::from_str::<Request>(&request)?;
    let response =
        handle_request(request, wmclient, command_runner).unwrap_or_else(|err| Response::Error(err.to_string()));

    // Response
    reader
        .into_inner()
        .write_all(serde_json::to_string(&response)?.as_bytes())?;

    Ok(())
}

fn handle_request(
    request: Request,
    wmclient: &mut WMClient,
    command_runner: &mut CommandRunner,
) -> anyhow::Result<Response> {
    match request {
        Request::ActiveWindow => Ok(Response::ActiveWindow {
            title: wmclient.current_window().unwrap_or_default(),
            wm_class: wmclient.current_application().unwrap_or_default(),
        }),

        Request::WindowList => Ok(Response::WindowList(wmclient.window_list()?)),

        Request::Run(command) => {
            command_runner.run(command);
            Ok(Response::Ok)
        }
    }
}
