use std::fs::read_dir;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use swayipc::Connection;

pub struct SwayClient {
    connection: Option<Connection>,
    supported: Option<bool>,
}

impl SwayClient {
    pub fn new() -> SwayClient {
        SwayClient {
            connection: None,
            supported: None,
        }
    }

    pub fn supported(&mut self) -> bool {
        match self.supported {
            Some(supported) => supported,
            None => {
                self.supported = Some(false);
                if let Some(socket) = find_socket() {
                    if let Ok(unix_stream) = UnixStream::connect(socket) {
                        self.connection = Some(Connection(unix_stream));
                        self.supported = Some(true);
                    }
                }
                self.supported.unwrap()
            }
        }
    }

    pub fn current_wm_class(&mut self) -> Option<String> {
        if !self.supported() {
            return None;
        }

        let connection = match &mut self.connection {
            Some(connection) => connection,
            None => return None,
        };

        if let Ok(node) = connection.get_tree() {
            if let Some(node) = node.find_focused(|n| n.focused) {
                return node.app_id
            }
        }
        None
    }
}

// e.g. "/run/user/1000/sway-ipc.1000.2575.sock"
fn find_socket() -> Option<String> {
    let uid = 1000; // Assume a first nornal Linux user
    if let Some(run_user) = read_dir(format!("/run/user/{}", uid)).as_mut().ok() {
        while let Some(entry) = run_user.next() {
            let path = entry.ok()?.path();
            if let Some(fname) = path.file_name() {
                if fname.as_bytes().starts_with(b"sway-ipc.") {
                    if let Ok(path) = path.into_os_string().into_string() {
                        return Some(path)
                    }
                }
            }
        }
    }
    None
}
