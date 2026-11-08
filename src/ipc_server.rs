#[cfg(any(target_os = "linux", target_os = "android"))]
use nix::sys::socket::sockopt::PeerCredentials;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

// ── Protocol ────────────────────────────────────────────────────────────────
//
// Line-based text protocol in the spirit of dotool: each command is a single
// line "<verb> [arg]\n". The server replies with a single line per command.
//
// Commands:
//   set-mode MODE      Switch the active keymap mode. → "ok"
//   get-mode           Query the active mode. → "MODE"
//   ping               Liveness check. → "pong"
//
// Replies: "ok\n", "MODE\n", "pong\n", "err REASON\n".

pub const MAX_LINE_LEN: usize = 256;
const MAX_MODE_LEN: usize = 64;
const MAX_CONNECTIONS: usize = 64;
const ACCEPT_BACKOFF: Duration = Duration::from_millis(500);

/// Commands that must be applied by the main loop (they touch
/// EventHandler / ActionDispatcher / MainAction, which the IPC server must not own).
#[derive(Debug, Clone, PartialEq)]
pub enum IpcCommand {
    SetMode(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    Ok,
    Mode(String),
    Pong,
    Err(String),
}

impl Reply {
    pub fn encode(&self) -> String {
        match self {
            Reply::Ok => "ok\n".to_string(),
            // Config/keymap modes need not satisfy the IPC set-mode syntax.
            // The mode name is the whole reply; clients tell replies apart by
            // order, and a failure is any line starting with `err`.
            Reply::Mode(mode) => format!("{}\n", escape_reply_text(mode)),
            Reply::Pong => "pong\n".to_string(),
            Reply::Err(reason) => format!("err {}\n", escape_reply_text(reason)),
        }
    }
}

/// Keep free-form reply text (`MODE` and `REASON`) on a single line.
/// Backslashes are escaped first, so a literal `\n` stays `\\n` instead of
/// looking like the line break that terminates the reply.
fn escape_reply_text(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\r', "\\r").replace('\n', "\\n")
}

fn valid_mode_name(mode: &str) -> bool {
    !mode.is_empty()
        && mode.len() <= MAX_MODE_LEN
        && mode.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Pure command handling, testable without a socket.
/// Returns the command to apply in the main loop (if any) and the reply.
pub fn handle_command(line: &str, current_mode: &str) -> (Option<IpcCommand>, Reply) {
    let line = line.trim();
    let (verb, arg) = match line.split_once(' ') {
        Some((verb, arg)) => (verb, arg.trim()),
        None => (line, ""),
    };
    match verb {
        "ping" => (None, Reply::Pong),
        "get-mode" => (None, Reply::Mode(current_mode.to_string())),
        "set-mode" => {
            if !valid_mode_name(arg) {
                return (None, Reply::Err("invalid mode name".to_string()));
            }
            (Some(IpcCommand::SetMode(arg.to_string())), Reply::Ok)
        }
        "" => (None, Reply::Err("empty command".to_string())),
        _ => (None, Reply::Err("unknown command".to_string())),
    }
}

// ── Transport ───────────────────────────────────────────────────────────────

/// Default socket location: $XDG_RUNTIME_DIR (per-user, 0700), with a
/// uid-suffixed temp dir fallback.
pub fn default_socket_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return Path::new(&dir).join("xremap.sock");
        }
    }
    let uid = nix::unistd::getuid();
    std::env::temp_dir().join(format!("xremap-{uid}.sock"))
}

/// Unparsed lines from one connection. Parsing and execution belong to the
/// main loop: a later line must observe the effects of an earlier command.
/// The cloned stream keeps the connection usable even when a config reload
/// replaces the listener.
pub struct IpcBatch {
    pub lines: Vec<Result<String, String>>,
    stream: UnixStream,
    close_after_reply: bool,
}

impl IpcBatch {
    pub fn reply(&mut self, reply: &Reply) -> std::io::Result<()> {
        let result = self.stream.write_all(reply.encode().as_bytes());
        if result.is_err() {
            // Do not leave a partially written reply on a reusable connection.
            let _ = self.stream.shutdown(std::net::Shutdown::Both);
        }
        result
    }
}

impl Drop for IpcBatch {
    fn drop(&mut self) {
        if self.close_after_reply {
            let _ = self.stream.shutdown(std::net::Shutdown::Both);
        }
    }
}

pub struct IpcServer {
    listener: UnixListener,
    path: PathBuf,
    identity: (u64, u64, i64, i64),
    accept_retry_at: Option<Instant>,
    connections: HashMap<RawFd, UnixStream>,
    buffers: HashMap<RawFd, Vec<u8>>,
}

fn socket_identity(meta: &std::fs::Metadata) -> (u64, u64, i64, i64) {
    (meta.dev(), meta.ino(), meta.ctime(), meta.ctime_nsec())
}

/// Never unlink an ordinary file, symlink, foreign socket, or live listener.
fn remove_stale_socket(path: &Path) -> anyhow::Result<()> {
    use nix::sys::socket::{connect, socket, AddressFamily, SockFlag, SockType, UnixAddr};
    let meta = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    anyhow::ensure!(meta.file_type().is_socket(), "IPC path already exists and is not a socket: {}", path.display());
    anyhow::ensure!(
        meta.uid() == nix::unistd::geteuid().as_raw(),
        "IPC socket belongs to another user: {}",
        path.display()
    );
    let probe = socket(AddressFamily::Unix, SockType::Stream, SockFlag::SOCK_NONBLOCK, None)?;
    match connect(probe.as_raw_fd(), &UnixAddr::new(path)?) {
        Err(nix::errno::Errno::ECONNREFUSED) => {
            let current = std::fs::symlink_metadata(path)?;
            anyhow::ensure!(
                socket_identity(&current) == socket_identity(&meta),
                "IPC socket changed while checking ownership"
            );
            std::fs::remove_file(path)?;
            Ok(())
        }
        Err(nix::errno::Errno::ENOENT) => Ok(()),
        _ => anyhow::bail!("IPC socket is in use or cannot be safely removed: {}", path.display()),
    }
}

impl IpcServer {
    pub fn new(path: &Path) -> anyhow::Result<Self> {
        remove_stale_socket(path)?;
        let listener = UnixListener::bind(path)?;
        // The socket is owner-only; connection peers are additionally checked
        // with SO_PEERCRED.
        std::fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        anyhow::ensure!(
            (listener.as_raw_fd() as usize) < nix::sys::select::FD_SETSIZE,
            "IPC listener fd exceeds select limit"
        );
        let identity = socket_identity(&std::fs::symlink_metadata(path)?);
        Ok(Self {
            listener,
            path: path.to_path_buf(),
            identity,
            accept_retry_at: None,
            connections: HashMap::new(),
            buffers: HashMap::new(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn holds_fd(&self, fd: RawFd) -> bool {
        fd == self.listener.as_raw_fd() || self.connections.contains_key(&fd)
    }

    /// All fds the server wants select(2)ed on: the listener plus connections.
    pub fn fds(&self) -> impl Iterator<Item = BorrowedFd<'_>> {
        std::iter::once(self.listener.as_fd())
            .filter(|_| self.accept_retry_delay().is_none())
            .chain(self.connections.values().map(AsFd::as_fd))
    }

    /// Wake select when accept can be retried; a resource error must not cause
    /// a hot readable-listener loop or prevent physical input processing.
    pub fn accept_retry_delay(&self) -> Option<Duration> {
        self.accept_retry_at
            .and_then(|at| at.checked_duration_since(Instant::now()))
    }

    fn check_peer(stream: &UnixStream) -> anyhow::Result<()> {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            let cred = nix::sys::socket::getsockopt(&stream, PeerCredentials)?;
            let euid = nix::unistd::getuid();
            // A root peer is always accepted. Otherwise the peer must run as
            // the same user as xremap. When xremap itself runs as root this
            // comparison admits every uid, so access control then rests on the
            // socket's 0600 mode and its parent directory, not on peer
            // credentials.
            if cred.uid() != 0 && euid.as_raw() != 0 && cred.uid() != euid.as_raw() {
                anyhow::bail!("rejected peer uid {}", cred.uid());
            }
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            // No SO_PEERCRED here: access is restricted by the 0600 socket.
            let _ = stream;
        }
        Ok(())
    }

    /// Read and frame lines without parsing them or acknowledging execution.
    /// Each caller must execute a line before parsing/replying to the next.
    pub fn handle_readable(&mut self, fd: RawFd) -> Option<IpcBatch> {
        if fd == self.listener.as_raw_fd() {
            self.accept();
            return None;
        }
        let stream = self.connections.get_mut(&fd)?;
        let mut chunk = [0u8; 4096];
        let n = match stream.read(&mut chunk) {
            Ok(0) => {
                self.drop_connection(fd);
                return None;
            }
            Ok(n) => n,
            Err(err) if matches!(err.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted) => {
                return None;
            }
            Err(err) => {
                eprintln!("xremap ipc: connection read failed: {err}");
                self.drop_connection(fd);
                return None;
            }
        };
        let reply_stream = match stream.try_clone() {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("xremap ipc: failed to retain reply stream: {err}");
                self.drop_connection(fd);
                return None;
            }
        };
        self.buffers.entry(fd).or_default().extend_from_slice(&chunk[..n]);
        let buffer = self.buffers.get_mut(&fd).unwrap();
        let mut lines = Vec::new();
        let mut close_after_reply = false;
        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
            if pos > MAX_LINE_LEN {
                lines.push(Err("line too long".to_string()));
                close_after_reply = true;
                break;
            }
            let mut line: Vec<u8> = buffer.drain(..=pos).collect();
            line.pop();
            lines.push(String::from_utf8(line).map_err(|_| "invalid UTF-8".to_string()));
        }
        if !close_after_reply && buffer.len() > MAX_LINE_LEN {
            lines.push(Err("line too long".to_string()));
            close_after_reply = true;
        }
        if close_after_reply {
            self.drop_connection(fd);
        }
        if lines.is_empty() {
            None
        } else {
            Some(IpcBatch {
                lines,
                stream: reply_stream,
                close_after_reply,
            })
        }
    }

    fn accept(&mut self) {
        self.accept_retry_at = None;
        // Bound work and descriptors even when a client keeps reconnecting.
        for _ in 0..MAX_CONNECTIONS {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if self.connections.len() >= MAX_CONNECTIONS
                        || (stream.as_raw_fd() as usize) >= nix::sys::select::FD_SETSIZE
                    {
                        continue;
                    }
                    if let Err(err) = Self::check_peer(&stream) {
                        eprintln!("xremap ipc: {err}");
                        continue;
                    }
                    if let Err(err) = stream.set_nonblocking(true) {
                        eprintln!("xremap ipc: failed to configure connection: {err}");
                        continue;
                    }
                    self.buffers.insert(stream.as_raw_fd(), Vec::new());
                    self.connections.insert(stream.as_raw_fd(), stream);
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(err) => {
                    eprintln!("xremap ipc: accept failed, will retry: {err}");
                    self.accept_retry_at = Some(Instant::now() + ACCEPT_BACKOFF);
                    break;
                }
            }
        }
    }

    fn drop_connection(&mut self, fd: RawFd) {
        self.connections.remove(&fd);
        self.buffers.remove(&fd);
    }
}

impl AsFd for IpcServer {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.listener.as_fd()
    }
}

impl Drop for IpcServer {
    fn drop(&mut self) {
        if let Ok(meta) = std::fs::symlink_metadata(&self.path) {
            if meta.file_type().is_socket() && socket_identity(&meta) == self.identity {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SocketDir(PathBuf);

    impl SocketDir {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("xremap-socket-{}-{:x}", std::process::id(), fastrand::u64(..)));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for SocketDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn live_socket_and_ordinary_files_are_not_unlinked() {
        let dir = SocketDir::new();
        let path = dir.0.join("control.sock");
        let server = IpcServer::new(&path).unwrap();
        assert!(IpcServer::new(&path).is_err());
        assert!(UnixStream::connect(&path).is_ok());
        drop(server);
        std::fs::write(&path, "keep this file").unwrap();
        assert!(IpcServer::new(&path).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep this file");
    }

    #[test]
    fn stale_owned_socket_can_be_replaced() {
        let dir = SocketDir::new();
        let path = dir.0.join("stale.sock");
        drop(UnixListener::bind(&path).unwrap());
        let server = IpcServer::new(&path).unwrap();
        assert!(UnixStream::connect(&path).is_ok());
        drop(server);
        assert!(!path.exists());
    }

    #[test]
    fn drop_leaves_a_replaced_path_alone() {
        let dir = SocketDir::new();
        let path = dir.0.join("control.sock");
        let server = IpcServer::new(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::write(&path, "replacement").unwrap();
        drop(server);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "replacement");
    }

    #[test]
    fn connections_and_accept_retry_are_bounded() {
        let dir = SocketDir::new();
        let path = dir.0.join("control.sock");
        let mut server = IpcServer::new(&path).unwrap();
        let _clients: Vec<_> = (0..MAX_CONNECTIONS + 1)
            .map(|_| UnixStream::connect(&path).unwrap())
            .collect();
        server.accept();
        server.accept();
        assert_eq!(server.connections.len(), MAX_CONNECTIONS);
        server.accept_retry_at = Some(Instant::now() + ACCEPT_BACKOFF);
        assert!(server.accept_retry_delay().is_some());
        assert!(!server.fds().any(|fd| fd.as_raw_fd() == server.listener.as_raw_fd()));
        server.accept_retry_at = Some(Instant::now() - ACCEPT_BACKOFF);
        assert!(server.accept_retry_delay().is_none());
        assert!(server.fds().any(|fd| fd.as_raw_fd() == server.listener.as_raw_fd()));
    }

    #[test]
    fn framing_preserves_partial_lines_and_rejects_invalid_utf8() {
        let dir = SocketDir::new();
        let path = dir.0.join("control.sock");
        let mut server = IpcServer::new(&path).unwrap();
        let mut client = UnixStream::connect(&path).unwrap();
        server.accept();
        let fd = *server.connections.keys().next().unwrap();
        client.write_all(b"pi").unwrap();
        assert!(server.handle_readable(fd).is_none());
        client.write_all(b"ng\n\xff\nping\n").unwrap();
        let batch = server.handle_readable(fd).unwrap();
        assert_eq!(batch.lines, [Ok("ping".into()), Err("invalid UTF-8".into()), Ok("ping".into())]);
    }

    #[test]
    fn overlong_lines_are_replied_to_then_closed() {
        let dir = SocketDir::new();
        let path = dir.0.join("control.sock");
        let mut server = IpcServer::new(&path).unwrap();
        let mut client = UnixStream::connect(&path).unwrap();
        client.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        server.accept();
        let fd = *server.connections.keys().next().unwrap();
        client.write_all(&vec![b'a'; MAX_LINE_LEN + 1]).unwrap();
        client.write_all(b"\n").unwrap();
        let mut batch = server
            .handle_readable(fd)
            .or_else(|| server.handle_readable(fd))
            .unwrap();
        assert_eq!(batch.lines, [Err("line too long".into())]);
        batch.reply(&Reply::Err("line too long".into())).unwrap();
        drop(batch);
        let mut reply = String::new();
        client.read_to_string(&mut reply).unwrap();
        assert_eq!(reply, "err line too long\n");
        assert!(!server.holds_fd(fd));
    }

    /// Parse a command.
    fn cmd(line: &str, mode: &str) -> (Option<IpcCommand>, Reply) {
        handle_command(line, mode)
    }

    #[test]
    fn ping() {
        assert_eq!(cmd("ping", "insert"), (None, Reply::Pong));
    }

    #[test]
    fn get_mode() {
        assert_eq!(cmd("get-mode", "normal"), (None, Reply::Mode("normal".into())));
    }

    #[test]
    fn set_mode() {
        assert_eq!(cmd("set-mode normal", "insert"), (Some(IpcCommand::SetMode("normal".into())), Reply::Ok));
    }

    #[test]
    fn set_mode_trims_whitespace() {
        assert_eq!(cmd("  set-mode   visual  \n", "insert"), (Some(IpcCommand::SetMode("visual".into())), Reply::Ok));
    }

    #[test]
    fn set_mode_rejects_bad_names() {
        for arg in ["", "with space", " Semi；colon", &"x".repeat(MAX_MODE_LEN + 1)] {
            let line = format!("set-mode {arg}");
            assert_eq!(cmd(&line, "insert").1, Reply::Err("invalid mode name".into()), "{arg:?}");
        }
    }

    #[test]
    fn unknown_and_empty() {
        assert_eq!(cmd("explode", "insert").1, Reply::Err("unknown command".into()));
        assert_eq!(cmd("", "insert").1, Reply::Err("empty command".into()));
    }

    #[test]
    fn replies_encode_to_single_lines() {
        assert_eq!(Reply::Ok.encode(), "ok\n");
        assert_eq!(Reply::Mode("normal".into()).encode(), "normal\n");
        assert_eq!(Reply::Mode("one\\ntwo\nthree\rfour".into()).encode(), "one\\\\ntwo\\nthree\\rfour\n");
        assert_eq!(Reply::Pong.encode(), "pong\n");
        assert_eq!(Reply::Err("boom".into()).encode(), "err boom\n");
        assert_eq!(Reply::Err("one\ntwo\rthree".into()).encode(), "err one\\ntwo\\rthree\n");
        // Backslashes are escaped first, so escaped-looking input stays literal.
        assert_eq!(Reply::Err("bad \\n here".into()).encode(), "err bad \\\\n here\n");
        assert_eq!(Reply::Mode("win\\path".into()).encode(), "win\\\\path\n");
    }

    #[test]
    fn end_to_end_socket() {
        let dir = std::env::temp_dir().join(format!("xremap-ipc-test-{}", std::process::id()));
        let path = dir.join("test.sock");
        std::fs::create_dir_all(&dir).unwrap();
        let mut server = IpcServer::new(&path).unwrap();

        let mut client = UnixStream::connect(&path).unwrap();
        // Accept the connection, then find the server-side fd of the connection
        // (different from the client-side fd in this single-process test).
        server.handle_readable(server.as_fd().as_raw_fd());
        let listener_fd = server.as_fd().as_raw_fd();
        let conn_fd = server
            .fds()
            .map(|fd| fd.as_raw_fd())
            .find(|fd| *fd != listener_fd)
            .unwrap();

        client.write_all(b"set-mode normal\nget-mode\nping\n").unwrap();
        let mut batch = server.handle_readable(conn_fd).unwrap();
        // Framing must not acknowledge commands that have not executed yet.
        client.set_nonblocking(true).unwrap();
        let mut buf = [0u8; 64];
        assert_eq!(client.read(&mut buf).unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
        client.set_nonblocking(false).unwrap();

        let mut mode = "insert".to_string();
        for line in std::mem::take(&mut batch.lines) {
            let (command, reply) = cmd(&line.unwrap(), &mode);
            if let Some(IpcCommand::SetMode(new_mode)) = command {
                mode = new_mode;
            }
            batch.reply(&reply).unwrap();
        }
        let mut output = [0u8; 15];
        client.read_exact(&mut output).unwrap();
        assert_eq!(&output, b"ok\nnormal\npong\n");
        drop(batch);

        drop(client);
        drop(server);
        assert!(!path.exists(), "socket file should be removed on drop");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
