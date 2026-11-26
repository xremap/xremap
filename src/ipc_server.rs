#[cfg(any(target_os = "linux", target_os = "android"))]
use nix::sys::socket::sockopt::PeerCredentials;
use nix::sys::socket::{connect, socket, AddressFamily, SockFlag, SockType, UnixAddr};
use nix::sys::time::{TimeVal, TimeValLike};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::action::Action;
use crate::config::key::parse_key;
use crate::config::key_combo::{parse_modifier_alias, Modifier};
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue};
use evdev::KeyCode as Key;
use std::rc::Rc;

// ── Protocol ────────────────────────────────────────────────────────────────
//
// One command per line: a single JSON object (JSON Lines). Requests are
// deserialized with serde, so the server keeps no hand-written parsing.
// The server replies with a single plain-text line per command.
//
// Requests:
//   {"command":"set-mode","mode":MODE}   Switch the active keymap mode.
//   {"command":"get-mode"}               Query the active mode.
//   {"command":"ping"}                   Liveness check.
//
//   Injection (gated behind `ipc.inject: true`):
//   {"command":"key","scope":"raw","device":"my-kbd","chords":["ctrl+a","Down*3"]}
//       scope: raw (default) emits straight to the app; remap feeds the chords
//       through the normal remapping pipeline (experimental_map, modmap and
//       keymap), exactly like a physical key.
//       A chord is `MODIFIER+...+KEY` with an optional `*N` repeat suffix.
//   {"command":"keydown","chords":["shift"]} / {"command":"keyup",...}
//       Only the press / only the release half.
//   {"command":"sleep","milliseconds":100}
//       Pause command processing for 1..=5000 ms.
//
// Replies: "ok\n", "MODE\n", "pong\n", "err REASON\n".

pub const MAX_LINE_LEN: usize = 16384;
const MAX_MODE_LEN: usize = 64;
const MAX_CHORDS: usize = 16;
const MAX_SLEEP_MS: u64 = 5000;
const MAX_CONNECTIONS: usize = 64;
const ACCEPT_BACKOFF: Duration = Duration::from_millis(500);

/// Whether injected keys bypass remapping or go through it.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum InjectScope {
    /// Straight to the app; no rule applies. Same semantics as dotool.
    Raw,
    /// Through the normal remapping pipeline (experimental_map → modmap →
    /// keymap), exactly like a physical key. Whether rules apply is still
    /// controllable with `mode`, `device:` and the usual conditions.
    Remap,
}

impl InjectScope {
    #[cfg(test)]
    fn parse(word: &str) -> Option<InjectScope> {
        match word {
            "raw" => Some(InjectScope::Raw),
            "remap" => Some(InjectScope::Remap),
            _ => None,
        }
    }
}

/// A parsed `MODIFIER+...+KEY` chord. Modifiers are emitted around the key.
#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    pub modifiers: Vec<Key>,
    pub key: Key,
}

/// Concrete key for a config modifier: the left-hand key for the matching
/// aliases, the exact key for `Modifier::Key`.
fn modifier_key(modifier: &Modifier) -> Key {
    match modifier {
        Modifier::Shift => Key::KEY_LEFTSHIFT,
        Modifier::Control => Key::KEY_LEFTCTRL,
        Modifier::Alt => Key::KEY_LEFTALT,
        Modifier::Windows => Key::KEY_LEFTMETA,
        Modifier::Key(key) => *key,
    }
}

/// Parse one chord part as a modifier, reusing the config parser: the same
/// aliases (S/C/A/M/SUPER/WIN/..., case-insensitive) and exact keys
/// (KEY_RIGHTALT, CODE_...) as `modmap`/`keymap` source modifiers.
fn parse_chord_modifier(part: &str) -> Option<Key> {
    parse_modifier_alias(part)
        .map(|modifier| modifier_key(&modifier))
        .or_else(|| parse_key(part).ok())
}

impl Chord {
    /// `ctrl+shift+a` → modifiers [LCTRL, LSHIFT], key A. Key names go through
    /// the same `parse_key` used by the config (F13, KEY_F13, CODE_191, ...).
    /// A lone modifier name (`shift`) is the left-hand key, so `keydown shift`
    /// can hold a modifier that survives across commands.
    pub fn parse(input: &str) -> Result<Chord, String> {
        let parts: Vec<&str> = input.split('+').collect();
        let (key_part, modifier_parts) = parts.split_last().ok_or("empty chord")?;
        let mut modifiers = Vec::new();
        for part in modifier_parts {
            let key = parse_chord_modifier(part).ok_or_else(|| format!("unknown modifier: {part}"))?;
            modifiers.push(key);
        }
        let key = match parse_key(key_part) {
            Ok(key) => key,
            // The config rejects ambiguous modifier names as keys; for a
            // lone modifier chord the left-hand key is the intended one.
            Err(err) if modifier_parts.is_empty() => parse_modifier_alias(key_part)
                .map(|modifier| modifier_key(&modifier))
                .ok_or_else(|| format!("invalid key: {err}"))?,
            Err(err) => return Err(format!("invalid key: {err}")),
        };
        Ok(Chord { modifiers, key })
    }

    /// Parse a chord with an optional `*N` repeat suffix (`Down*3`).
    pub fn parse_with_repeat(input: &str) -> Result<Vec<Chord>, String> {
        let (base, repeat) = match input.rsplit_once('*') {
            Some((base, count)) => {
                let count: usize = count.parse().map_err(|_| format!("invalid repeat count: {count}"))?;
                if count == 0 || count > MAX_CHORDS {
                    return Err(format!("repeat count must be 1..={MAX_CHORDS}"));
                }
                (base, count)
            }
            None => (input, 1),
        };
        let chord = Chord::parse(base)?;
        Ok(vec![chord; repeat])
    }

    /// Modifier presses, key press/release, modifier releases in reverse order.
    /// `down`/`up` select which halves are produced (keydown/keyup commands).
    pub fn to_actions(&self, down: bool, up: bool) -> Vec<Action> {
        self.to_event_pairs(down, up)
            .into_iter()
            .map(|(key, value)| Action::KeyEvent(KeyEvent::new(key, value)))
            .collect()
    }

    /// Same sequence as `to_actions`, but as pipeline events for scopes that
    /// go through EventHandler. `device` is the synthetic IPC device identity.
    pub fn to_events(&self, device: Rc<InputDeviceInfo>, down: bool, up: bool) -> Vec<Event> {
        self.to_event_pairs(down, up)
            .into_iter()
            .map(|(key, value)| Event::KeyEvent(device.clone(), KeyEvent::new(key, value)))
            .collect()
    }

    fn to_event_pairs(&self, down: bool, up: bool) -> Vec<(Key, KeyValue)> {
        let mut pairs = Vec::with_capacity(self.modifiers.len() * 2 + 2);
        if down {
            for modifier in &self.modifiers {
                pairs.push((*modifier, KeyValue::Press));
            }
            pairs.push((self.key, KeyValue::Press));
        }
        if up {
            pairs.push((self.key, KeyValue::Release));
            for modifier in self.modifiers.iter().rev() {
                pairs.push((*modifier, KeyValue::Release));
            }
        }
        pairs
    }
}

/// Which halves of the press/release cycle a `key`-family command emits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyPhases {
    /// keydown + keyup (the `key` command).
    Tap,
    /// keydown only (the `keydown` command).
    Down,
    /// keyup only (the `keyup` command).
    Up,
}

/// Commands that must be applied by the main loop (they touch
/// EventHandler / ActionDispatcher / MainAction, which the IPC server must not own).
#[derive(Debug, Clone, PartialEq)]
pub enum IpcCommand {
    SetMode(String),
    SendKey {
        scope: InjectScope,
        /// Simulated source device name; matches `device:` keymap conditions.
        device: Option<String>,
        chords: Vec<Chord>,
        phases: KeyPhases,
    },
    /// Pause command processing; maps to the existing Action::Delay.
    Sleep(std::time::Duration),
}

impl IpcCommand {
    #[cfg(test)]
    fn scope(&self) -> InjectScope {
        match self {
            IpcCommand::SendKey { scope, .. } => *scope,
            _ => panic!("not a SendKey"),
        }
    }
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
            // Errors may come from multiline config/parser diagnostics.
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

/// One request line, deserialized with serde.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(tag = "command", rename_all = "kebab-case", deny_unknown_fields)]
pub enum IpcRequest {
    Ping,
    GetMode,
    SetMode {
        mode: String,
    },
    /// Inject full key chords. `scope` defaults to `raw`.
    Key {
        #[serde(default)]
        scope: Option<InjectScope>,
        #[serde(default)]
        device: Option<String>,
        chords: Vec<String>,
    },
    /// Only the press half.
    Keydown {
        #[serde(default)]
        scope: Option<InjectScope>,
        #[serde(default)]
        device: Option<String>,
        chords: Vec<String>,
    },
    /// Only the release half.
    Keyup {
        #[serde(default)]
        scope: Option<InjectScope>,
        #[serde(default)]
        device: Option<String>,
        chords: Vec<String>,
    },
    /// Pause command processing.
    Sleep {
        milliseconds: u64,
    },
}

/// The part of a serde_json error after the line/column suffix; requests are
/// always single lines, so those positions carry no information.
fn describe_parse_error(err: serde_json::Error) -> String {
    let message = err.to_string();
    match message.find(" at line ") {
        Some(pos) => message[..pos].to_string(),
        None => message,
    }
}

fn injection_disabled() -> (Option<IpcCommand>, Reply) {
    (None, Reply::Err("injection disabled".into()))
}

/// Pure command handling, testable without a socket.
/// Returns the command to apply in the main loop (if any) and the reply.
pub fn handle_command(line: &str, current_mode: &str, inject_enabled: bool) -> (Option<IpcCommand>, Reply) {
    let request: IpcRequest = match serde_json::from_str(line) {
        Ok(request) => request,
        Err(err) => return (None, Reply::Err(describe_parse_error(err))),
    };
    match request {
        IpcRequest::Ping => (None, Reply::Pong),
        IpcRequest::GetMode => (None, Reply::Mode(current_mode.to_string())),
        IpcRequest::SetMode { mode } => {
            if !valid_mode_name(&mode) {
                return (None, Reply::Err("invalid mode name".to_string()));
            }
            (Some(IpcCommand::SetMode(mode)), Reply::Ok)
        }
        IpcRequest::Key { scope, device, chords } => {
            if !inject_enabled {
                return injection_disabled();
            }
            match parse_key_request(scope, device, &chords, KeyPhases::Tap) {
                Ok(command) => (Some(command), Reply::Ok),
                Err(reason) => (None, Reply::Err(reason)),
            }
        }
        IpcRequest::Keydown { scope, device, chords } => {
            if !inject_enabled {
                return injection_disabled();
            }
            match parse_key_request(scope, device, &chords, KeyPhases::Down) {
                Ok(command) => (Some(command), Reply::Ok),
                Err(reason) => (None, Reply::Err(reason)),
            }
        }
        IpcRequest::Keyup { scope, device, chords } => {
            if !inject_enabled {
                return injection_disabled();
            }
            match parse_key_request(scope, device, &chords, KeyPhases::Up) {
                Ok(command) => (Some(command), Reply::Ok),
                Err(reason) => (None, Reply::Err(reason)),
            }
        }
        IpcRequest::Sleep { milliseconds } => {
            if !inject_enabled {
                return injection_disabled();
            }
            if !(1..=MAX_SLEEP_MS).contains(&milliseconds) {
                return (None, Reply::Err(format!("sleep takes 1..={MAX_SLEEP_MS} milliseconds")));
            }
            (Some(IpcCommand::Sleep(Duration::from_millis(milliseconds))), Reply::Ok)
        }
    }
}

/// Validate the shared fields of `key`/`keydown`/`keyup`.
fn parse_key_request(
    scope: Option<InjectScope>,
    device: Option<String>,
    chords: &[String],
    phases: KeyPhases,
) -> Result<IpcCommand, String> {
    if chords.is_empty() {
        return Err("missing chords".to_string());
    }
    if let Some(device) = &device {
        if device.len() > MAX_MODE_LEN {
            return Err("device name too long".to_string());
        }
    }
    let mut parsed = Vec::new();
    for chord in chords {
        parsed.extend(Chord::parse_with_repeat(chord)?);
    }
    if parsed.len() > MAX_CHORDS {
        return Err(format!("too many chords (max {MAX_CHORDS})"));
    }
    Ok(IpcCommand::SendKey {
        scope: scope.unwrap_or(InjectScope::Raw),
        device,
        chords: parsed,
        phases,
    })
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

    /// All fds the server wants select(2)ed on: the listener plus connections.
    pub fn fds(&self) -> impl Iterator<Item = BorrowedFd<'_>> {
        std::iter::once(self.listener.as_fd())
            .filter(|_| self.accept_retry_delay().is_none())
            .chain(self.connections.values().map(AsFd::as_fd))
    }

    /// Wake select when accept can be retried; a resource error must not cause
    /// a hot readable-listener loop or prevent physical input processing.
    /// Snapshot the retry delay before collecting fds, so an expiring backoff
    /// cannot leave a select call with neither a listener nor a wakeup timeout.
    pub fn accept_retry_delay(&self) -> Option<TimeVal> {
        self.accept_retry_at
            .and_then(|at| at.checked_duration_since(Instant::now()))
            .map(|delay| TimeVal::microseconds(delay.as_micros() as i64))
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

        match self.listener.accept() {
            Ok((stream, _)) => {
                if self.connections.len() >= MAX_CONNECTIONS
                    || (stream.as_raw_fd() as usize) >= nix::sys::select::FD_SETSIZE
                {
                    return;
                }
                if let Err(err) = Self::check_peer(&stream) {
                    eprintln!("xremap ipc: {err}");
                    return;
                }
                if let Err(err) = stream.set_nonblocking(true) {
                    eprintln!("xremap ipc: failed to configure connection: {err}");
                    return;
                }
                self.buffers.insert(stream.as_raw_fd(), Vec::new());
                self.connections.insert(stream.as_raw_fd(), stream);
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
            Err(err) => {
                eprintln!("xremap ipc: accept failed, will retry: {err}");
                self.accept_retry_at = Some(Instant::now() + ACCEPT_BACKOFF);
            }
        }
    }

    fn drop_connection(&mut self, fd: RawFd) {
        self.connections.remove(&fd);
        self.buffers.remove(&fd);
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

    /// `parse_key` consults the global key-alias table, which is normally
    /// populated while parsing a config file. Set it once here so these tests
    /// do not depend on other tests having parsed a config first.
    fn init_aliases() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            crate::config::key_aliases::pre_load_key_aliases("", crate::config::ConfigFiletype::Yaml).unwrap();
        });
    }

    fn cmd(line: &str, mode: &str, inject: bool) -> (Option<IpcCommand>, Reply) {
        init_aliases();
        handle_command(line, mode, inject)
    }

    /// Build a one-line JSON request.
    fn request(command: &str, fields: &str) -> String {
        if fields.is_empty() {
            format!(r#"{{"command":"{command}"}}"#)
        } else {
            format!(r#"{{"command":"{command}",{fields}}}"#)
        }
    }

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
    fn live_socket_is_not_unlinked() {
        let dir = SocketDir::new();
        let path = dir.0.join("control.sock");
        let _server = IpcServer::new(&path).unwrap();
        assert!(IpcServer::new(&path).is_err());
        assert!(UnixStream::connect(&path).is_ok());
    }

    #[test]
    fn ordinary_file_is_not_unlinked() {
        let dir = SocketDir::new();
        let path = dir.0.join("control.sock");
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
        // A read returns at most 4096 bytes, so a line above the cap spans
        // several reads; only the last one can report it as overlong.
        let mut batch = loop {
            if let Some(batch) = server.handle_readable(fd) {
                break batch;
            }
        };
        assert_eq!(batch.lines, [Err("line too long".into())]);
        batch.reply(&Reply::Err("line too long".into())).unwrap();
        drop(batch);
        let mut reply = String::new();
        client.read_to_string(&mut reply).unwrap();
        assert_eq!(reply, "err line too long\n");
    }

    #[test]
    fn ping() {
        assert_eq!(cmd(&request("ping", ""), "insert", false), (None, Reply::Pong));
    }

    #[test]
    fn get_mode() {
        assert_eq!(cmd(&request("get-mode", ""), "normal", false), (None, Reply::Mode("normal".into())));
    }

    #[test]
    fn set_mode() {
        assert_eq!(
            cmd(&request("set-mode", r#""mode":"normal""#), "insert", false),
            (Some(IpcCommand::SetMode("normal".into())), Reply::Ok)
        );
    }

    #[test]
    fn parse_errors_are_reported_as_err_replies() {
        for line in [
            "",
            "ping",
            "{}",
            "[]",
            "{not json}",
            r#"{"command":"explode"}"#,
            r#"{"command":"set-mode"}"#,
            r#"{"command":"set-mode","mode":"x","extra":1}"#,
            r#"{"command":"sleep"}"#,
        ] {
            assert!(matches!(cmd(line, "insert", false).1, Reply::Err(_)), "{line:?}");
        }
        // Line/column positions are stripped; the reason stays on one line.
        assert_eq!(
            cmd(r#"{"command":"explode"}"#, "insert", false).1,
            Reply::Err("unknown variant `explode`, expected one of `ping`, `get-mode`, `set-mode`, `key`, `keydown`, `keyup`, `sleep`".into())
        );
    }

    #[test]
    fn set_mode_rejects_bad_names() {
        for mode in ["", "with space", " Semi；colon", &"x".repeat(MAX_MODE_LEN + 1)] {
            let line = serde_json::json!({"command": "set-mode", "mode": mode}).to_string();
            assert_eq!(cmd(&line, "insert", false).1, Reply::Err("invalid mode name".into()), "{mode:?}");
        }
    }

    #[test]
    fn key_disabled_by_default() {
        assert_eq!(
            cmd(&request("key", r#""chords":["f13"]"#), "insert", false).1,
            Reply::Err("injection disabled".into())
        );
        assert_eq!(
            cmd(&request("sleep", r#""milliseconds":100"#), "insert", false).1,
            Reply::Err("injection disabled".into())
        );
    }

    #[test]
    fn key_single_chord_defaults_to_raw() {
        assert_eq!(
            cmd(&request("key", r#""chords":["f13"]"#), "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: None,
                    chords: vec![Chord {
                        modifiers: vec![],
                        key: Key::KEY_F13
                    }],
                    phases: KeyPhases::Tap,
                }),
                Reply::Ok
            )
        );
    }

    #[test]
    fn key_scope_and_multiple_chords() {
        assert_eq!(
            cmd(&request("key", r#""scope":"remap","chords":["ctrl+shift+a","f13"]"#), "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Remap,
                    device: None,
                    chords: vec![
                        Chord {
                            modifiers: vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT],
                            key: Key::KEY_A
                        },
                        Chord {
                            modifiers: vec![],
                            key: Key::KEY_F13
                        }
                    ],
                    phases: KeyPhases::Tap,
                }),
                Reply::Ok
            )
        );
        // Only raw and remap exist; removed entry-layer scopes are rejected.
        assert!(matches!(
            cmd(&request("key", r#""scope":"expmap","chords":["x"]"#), "insert", true).1,
            Reply::Err(_)
        ));
    }

    #[test]
    fn key_device_option() {
        assert_eq!(
            cmd(&request("key", r#""device":"my-kbd","chords":["f13"]"#), "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: Some("my-kbd".into()),
                    chords: vec![Chord {
                        modifiers: vec![],
                        key: Key::KEY_F13
                    }],
                    phases: KeyPhases::Tap,
                }),
                Reply::Ok
            )
        );
        assert_eq!(
            cmd(
                &request("key", &format!(r#""device":"{}","chords":["f13"]"#, "x".repeat(MAX_MODE_LEN + 1))),
                "insert",
                true
            )
            .1,
            Reply::Err("device name too long".into())
        );
    }

    #[test]
    fn key_missing_chords() {
        assert_eq!(cmd(&request("key", r#""chords":[]"#), "insert", true).1, Reply::Err("missing chords".into()));
        assert_eq!(cmd(&request("keydown", r#""chords":[]"#), "insert", true).1, Reply::Err("missing chords".into()));
        assert_eq!(cmd(&request("keyup", r#""chords":[]"#), "insert", true).1, Reply::Err("missing chords".into()));
    }

    #[test]
    fn key_repeat_suffix() {
        let down = || Chord {
            modifiers: vec![],
            key: Key::KEY_DOWN,
        };
        assert_eq!(
            cmd(&request("key", r#""chords":["Down*3"]"#), "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: None,
                    chords: vec![down(), down(), down()],
                    phases: KeyPhases::Tap,
                }),
                Reply::Ok
            )
        );
        assert_eq!(
            cmd(&request("key", r#""chords":["Down*0"]"#), "insert", true).1,
            Reply::Err("repeat count must be 1..=16".into())
        );
        assert_eq!(
            cmd(&request("key", r#""chords":["Down*17"]"#), "insert", true).1,
            Reply::Err("repeat count must be 1..=16".into())
        );
    }

    #[test]
    fn keydown_keyup_phases() {
        let chord = vec![Chord {
            modifiers: vec![],
            key: Key::KEY_F13,
        }];
        for (command, phases) in [("keydown", KeyPhases::Down), ("keyup", KeyPhases::Up)] {
            assert_eq!(
                cmd(&request(command, r#""chords":["f13"]"#), "insert", true),
                (
                    Some(IpcCommand::SendKey {
                        scope: InjectScope::Raw,
                        device: None,
                        chords: chord.clone(),
                        phases,
                    }),
                    Reply::Ok
                )
            );
        }
    }

    #[test]
    fn lone_modifier_chord_uses_the_left_key() {
        // `keydown shift` holds LEFTSHIFT until a matching `keyup shift`.
        assert_eq!(
            cmd(&request("keydown", r#""chords":["shift"]"#), "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: None,
                    chords: vec![Chord {
                        modifiers: vec![],
                        key: Key::KEY_LEFTSHIFT
                    }],
                    phases: KeyPhases::Down,
                }),
                Reply::Ok
            )
        );
        // Modifier names elsewhere still require an explicit left/right key.
        assert!(matches!(cmd(&request("key", r#""chords":["ctrl+shift"]"#), "insert", true).1, Reply::Err(_)));
    }

    #[test]
    fn key_down_up_actions_halves() {
        init_aliases();
        let chord = Chord::parse("ctrl+a").unwrap();
        let keys = |actions: Vec<Action>| -> Vec<(Key, KeyValue)> {
            actions
                .iter()
                .map(|a| match a {
                    Action::KeyEvent(e) => (e.key, e.value),
                    _ => panic!("unexpected"),
                })
                .collect()
        };
        assert_eq!(
            keys(chord.to_actions(true, false)),
            vec![(Key::KEY_LEFTCTRL, KeyValue::Press), (Key::KEY_A, KeyValue::Press)]
        );
        assert_eq!(
            keys(chord.to_actions(false, true)),
            vec![(Key::KEY_A, KeyValue::Release), (Key::KEY_LEFTCTRL, KeyValue::Release)]
        );
    }

    #[test]
    fn key_scopes_are_raw_and_remap() {
        for scope in ["raw", "remap"] {
            let line = serde_json::json!({"command": "key", "scope": scope, "chords": ["x"]}).to_string();
            let (command, _) = cmd(&line, "insert", true);
            assert_eq!(command.unwrap().scope(), InjectScope::parse(scope).unwrap(), "{scope}");
        }
    }

    #[test]
    fn key_rejects_bad_chords() {
        for fields in [
            r#""chords":["foo+bar"]"#,
            r#""chords":["ctrl+"]"#,
            r#""chords":["warp"]"#,
        ] {
            let reply = cmd(&request("key", fields), "insert", true).1;
            assert!(matches!(reply, Reply::Err(_)), "{fields:?} → {reply:?}");
        }
    }

    #[test]
    fn chord_modifiers_reuse_the_config_parser() {
        // Same aliases and case-insensitivity as `modmap`/`keymap` sources.
        let chord = Chord::parse("S+a").unwrap();
        assert_eq!(chord.modifiers, vec![Key::KEY_LEFTSHIFT]);
        assert_eq!(chord.key, Key::KEY_A);
        assert_eq!(Chord::parse("super+f13").unwrap().modifiers, vec![Key::KEY_LEFTMETA]);
        assert_eq!(Chord::parse("control+alt+x").unwrap().modifiers, vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTALT]);
        // Exact keys work too, and a lone alias is the left-hand key.
        assert_eq!(Chord::parse("KEY_RIGHTALT+x").unwrap().modifiers, vec![Key::KEY_RIGHTALT]);
        assert_eq!(Chord::parse("shift").unwrap().key, Key::KEY_LEFTSHIFT);
    }

    #[test]
    fn chord_to_actions_press_release_order() {
        init_aliases();
        let chord = Chord::parse("ctrl+a").unwrap();
        let actions = chord.to_actions(true, true);
        let keys: Vec<(Key, KeyValue)> = actions
            .iter()
            .map(|action| match action {
                Action::KeyEvent(event) => (event.key, event.value),
                _ => panic!("unexpected action"),
            })
            .collect();
        assert_eq!(
            keys,
            vec![
                (Key::KEY_LEFTCTRL, KeyValue::Press),
                (Key::KEY_A, KeyValue::Press),
                (Key::KEY_A, KeyValue::Release),
                (Key::KEY_LEFTCTRL, KeyValue::Release),
            ]
        );
    }

    #[test]
    fn sleep_command() {
        assert_eq!(
            cmd(&request("sleep", r#""milliseconds":100"#), "insert", true),
            (Some(IpcCommand::Sleep(Duration::from_millis(100))), Reply::Ok)
        );
        assert_eq!(
            cmd(&request("sleep", r#""milliseconds":0"#), "insert", true).1,
            Reply::Err("sleep takes 1..=5000 milliseconds".into())
        );
        assert_eq!(
            cmd(&request("sleep", r#""milliseconds":5001"#), "insert", true).1,
            Reply::Err("sleep takes 1..=5000 milliseconds".into())
        );
    }

    #[test]
    fn unknown_and_empty() {
        assert!(matches!(cmd("explode", "insert", false).1, Reply::Err(_)));
        assert!(matches!(cmd("", "insert", false).1, Reply::Err(_)));
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
        server.handle_readable(server.listener.as_fd().as_raw_fd());
        let listener_fd = server.listener.as_fd().as_raw_fd();
        let conn_fd = server
            .fds()
            .map(|fd| fd.as_raw_fd())
            .find(|fd| *fd != listener_fd)
            .unwrap();

        client
            .write_all(
                b"{\"command\":\"set-mode\",\"mode\":\"normal\"}\n{\"command\":\"get-mode\"}\n{\"command\":\"ping\"}\n",
            )
            .unwrap();
        let mut batch = server.handle_readable(conn_fd).unwrap();
        // Framing must not acknowledge commands that have not executed yet.
        client.set_nonblocking(true).unwrap();
        let mut buf = [0u8; 64];
        assert_eq!(client.read(&mut buf).unwrap_err().kind(), std::io::ErrorKind::WouldBlock);
        client.set_nonblocking(false).unwrap();

        let mut mode = "insert".to_string();
        for line in std::mem::take(&mut batch.lines) {
            let (command, reply) = cmd(&line.unwrap(), &mode, false);
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
