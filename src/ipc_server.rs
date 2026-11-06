#[cfg(any(target_os = "linux", target_os = "android"))]
use nix::sys::socket::sockopt::PeerCredentials;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::time::{Duration, Instant};

use crate::action::Action;
use crate::config::key::parse_key;
use crate::device::InputDeviceInfo;
use crate::event::{Event, KeyEvent, KeyValue};
use evdev::KeyCode as Key;
use std::rc::Rc;

// ── Protocol ────────────────────────────────────────────────────────────────
//
// Line-based text protocol in the spirit of dotool: each command is a single
// line "<verb> [arg]\n". The server replies with a single line per command.
//
// Commands:
//   set-mode MODE      Switch the active keymap mode. → "ok"
//   get-mode           Query the active mode. → "MODE"
//   reload-config      Reload the config files. → "ok"
//   ping               Liveness check. → "pong"
//
// Replies: "ok\n", "MODE\n", "pong\n", "err REASON\n".

pub const MAX_LINE_LEN: usize = 256;
const MAX_MODE_LEN: usize = 64;
const MAX_CHORDS: usize = 16;
const MAX_SLEEP_MS: u64 = 5000;
const MAX_CONNECTIONS: usize = 64;
const ACCEPT_BACKOFF: Duration = Duration::from_millis(500);

/// How deep into the processing pipeline an injected key enters.
/// From the entering layer, the key flows on to the app through the
/// remaining layers. The pipeline is:
/// **experimental_map (expmap) operators → modmap → keymap → app**.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InjectScope {
    /// Straight to the app; no layer applies. Same semantics as dotool.
    Raw,
    /// Enter at keymap: → keymap → app. Modmap is skipped.
    Keymap,
    /// Enter at modmap: → modmap → keymap → app. expmap operators are skipped.
    Modmap,
    /// Enter at the experimental_map operators:
    /// → expmap → modmap → keymap → app.
    /// Indistinguishable from a physical keypress.
    Expmap,
}

impl InjectScope {
    fn parse(word: &str) -> Option<InjectScope> {
        match word {
            "raw" => Some(InjectScope::Raw),
            "modmap" => Some(InjectScope::Modmap),
            "keymap" => Some(InjectScope::Keymap),
            "expmap" | "experimental_map" => Some(InjectScope::Expmap),
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

fn modifier_key(name: &str) -> Option<Key> {
    match name {
        "ctrl" | "control" => Some(Key::KEY_LEFTCTRL),
        "alt" => Some(Key::KEY_LEFTALT),
        "shift" => Some(Key::KEY_LEFTSHIFT),
        "super" | "meta" | "win" => Some(Key::KEY_LEFTMETA),
        "altgr" => Some(Key::KEY_RIGHTALT),
        _ => None,
    }
}

impl Chord {
    /// `ctrl+shift+a` → modifiers [LCTRL, LSHIFT], key A. Key names go through
    /// the same `parse_key` used by the config (F13, KEY_F13, CODE_191, ...).
    pub fn parse(input: &str) -> Result<Chord, String> {
        let parts: Vec<&str> = input.split('+').collect();
        let (key_part, modifier_parts) = parts.split_last().ok_or("empty chord")?;
        let mut modifiers = Vec::new();
        for part in modifier_parts {
            let key = modifier_key(part)
                .ok_or_else(|| format!("unknown modifier: {part}"))?;
            modifiers.push(key);
        }
        let key = parse_key(key_part).map_err(|err| format!("invalid key: {err}"))?;
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

/// Commands that must be applied by the main loop (they touch
/// EventHandler / ActionDispatcher / MainAction, which the IPC server must not own).
impl IpcCommand {
    #[cfg(test)]
    fn scope(&self) -> InjectScope {
        match self {
            IpcCommand::SendKey { scope, .. } => *scope,
            _ => panic!("not a SendKey"),
        }
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

#[derive(Debug, Clone, PartialEq)]
pub enum IpcCommand {
    SetMode(String),
    ReloadConfig,
    SendKey {
        scope: InjectScope,
        /// Simulated source device name; matches `device:` keymap conditions.
        device: Option<String>,
        chords: Vec<Chord>,
        phases: KeyPhases,
    },
    /// Pause command processing; maps to the existing Action::Delay.
    Sleep(std::time::Duration),
    /// Scroll by `amount` lines (REL_WHEEL) or columns (REL_HWHEEL).
    Wheel { horizontal: bool, amount: i32 },
    /// Move the pointer by x/y units, sent as one REL_X/REL_Y batch.
    MoveMouse { x: i32, y: i32 },
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
pub fn handle_command(line: &str, current_mode: &str, inject_enabled: bool) -> (Option<IpcCommand>, Reply) {
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
        "reload-config" => (Some(IpcCommand::ReloadConfig), Reply::Ok),
        "key" | "keydown" | "keyup" => {
            if !inject_enabled {
                return (None, Reply::Err("injection disabled".to_string()));
            }
            let phases = match verb {
                "key" => KeyPhases::Tap,
                "keydown" => KeyPhases::Down,
                _ => KeyPhases::Up,
            };
            match parse_key_command(arg) {
                Ok((scope, device, chords)) => (
                    Some(IpcCommand::SendKey { scope, device, chords, phases }),
                    Reply::Ok,
                ),
                Err(reason) => (None, Reply::Err(reason)),
            }
        }
        "click" => {
            if !inject_enabled {
                return (None, Reply::Err("injection disabled".to_string()));
            }
            match parse_click_command(arg) {
                Ok(chords) => (
                    Some(IpcCommand::SendKey { scope: InjectScope::Raw, device: None, chords, phases: KeyPhases::Tap }),
                    Reply::Ok,
                ),
                Err(reason) => (None, Reply::Err(reason)),
            }
        }
        "wheel" | "hwheel" => {
            if !inject_enabled {
                return (None, Reply::Err("injection disabled".to_string()));
            }
            match arg.trim().parse::<i32>() {
                Ok(amount) => (
                    Some(IpcCommand::Wheel { horizontal: verb == "hwheel", amount }),
                    Reply::Ok,
                ),
                Err(_) => (None, Reply::Err(format!("{verb} takes an integer amount"))),
            }
        }
        "mousemove" => {
            if !inject_enabled {
                return (None, Reply::Err("injection disabled".to_string()));
            }
            match parse_mousemove_command(arg) {
                Ok((x, y)) => (Some(IpcCommand::MoveMouse { x, y }), Reply::Ok),
                Err(reason) => (None, Reply::Err(reason)),
            }
        }
        "sleep" => {
            if !inject_enabled {
                return (None, Reply::Err("injection disabled".to_string()));
            }
            let Some(ms) = arg.parse::<u64>().ok().filter(|ms| (1..=MAX_SLEEP_MS).contains(ms)) else {
                return (None, Reply::Err(format!("sleep takes 1..={MAX_SLEEP_MS} milliseconds")));
            };
            (Some(IpcCommand::Sleep(std::time::Duration::from_millis(ms))), Reply::Ok)
        }
        "" => (None, Reply::Err("empty command".to_string())),
        _ => (None, Reply::Err("unknown command".to_string())),
    }
}

/// `key [--device NAME] [SCOPE] CHORD...` — both the device and the scope
/// are optional (default device: xremap-ipc, default scope: raw).
/// A chord may carry a `*N` repeat suffix (`Down*3`).
fn parse_key_command(arg: &str) -> Result<(InjectScope, Option<String>, Vec<Chord>), String> {
    let mut words = arg.split_whitespace();
    let mut scope = InjectScope::Raw;
    let mut device: Option<String> = None;
    let mut chords: Vec<Chord> = Vec::new();
    while let Some(word) = words.next() {
        if word == "--device" {
            let name = words.next().ok_or("--device requires a name")?;
            if name.len() > MAX_MODE_LEN {
                return Err("device name too long".to_string());
            }
            device = Some(name.to_string());
        } else if chords.is_empty() && scope == InjectScope::Raw && InjectScope::parse(word).is_some() {
            // The scope keyword is only recognized before the first chord.
            scope = InjectScope::parse(word).unwrap();
        } else {
            chords.extend(Chord::parse_with_repeat(word)?);
        }
    }
    if chords.is_empty() {
        return Err("missing chords".to_string());
    }
    if chords.len() > MAX_CHORDS {
        return Err(format!("too many chords (max {MAX_CHORDS})"));
    }
    Ok((scope, device, chords))
}

/// `click BUTTON...` — button names map to the evdev BTN_* keys, so the
/// command is a plain raw `key` tap and reuses the SendKey execution path.
fn parse_click_command(arg: &str) -> Result<Vec<Chord>, String> {
    let chords: Vec<Chord> = arg
        .split_whitespace()
        .map(|button| {
            let key = match button {
                "left" | "1" => "btn_left",
                "middle" | "2" => "btn_middle",
                "right" | "3" => "btn_right",
                _ => return Err(format!("unknown button: {button} (left/middle/right)")),
            };
            Chord::parse(key)
        })
        .collect::<Result<_, _>>()?;
    if chords.is_empty() {
        return Err("click requires a button (left/middle/right)".to_string());
    }
    Ok(chords)
}

/// `mousemove X Y` — two integers, REL_X and REL_Y values.
fn parse_mousemove_command(arg: &str) -> Result<(i32, i32), String> {
    let mut values = arg.split_whitespace().map(|word| {
        word.parse::<i32>().map_err(|_| format!("invalid coordinate: {word}"))
    });
    let (x, y) = (values.next().transpose()?, values.next().transpose()?);
    match (x, y, values.next()) {
        (Some(x), Some(y), None) => Ok((x, y)),
        (None, _, _) | (_, None, _) => Err("mousemove takes X and Y coordinates".to_string()),
        (_, _, Some(_)) => Err("mousemove takes exactly two coordinates".to_string()),
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
/// The cloned stream lets a reload reply even if it changes the socket path.
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
    anyhow::ensure!(meta.uid() == nix::unistd::geteuid().as_raw(), "IPC socket belongs to another user: {}", path.display());
    let probe = socket(AddressFamily::Unix, SockType::Stream, SockFlag::SOCK_NONBLOCK, None)?;
    match connect(probe.as_raw_fd(), &UnixAddr::new(path)?) {
        Err(nix::errno::Errno::ECONNREFUSED) => {
            let current = std::fs::symlink_metadata(path)?;
            anyhow::ensure!(socket_identity(&current) == socket_identity(&meta), "IPC socket changed while checking ownership");
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
        anyhow::ensure!((listener.as_raw_fd() as usize) < nix::sys::select::FD_SETSIZE, "IPC listener fd exceeds select limit");
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
        self.accept_retry_at.and_then(|at| at.checked_duration_since(Instant::now()))
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
            Some(IpcBatch { lines, stream: reply_stream, close_after_reply })
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

    /// `parse_key` consults the global key-alias table, which is normally
    /// populated while parsing a config file. Set it once here so these tests
    /// do not depend on other tests having parsed a config first.
    fn init_aliases() {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            crate::config::key_aliases::pre_load_key_aliases("", crate::config::ConfigFiletype::Yaml).unwrap();
        });
    }

    /// Parse a command after initializing the aliases used by key parsing.
    fn cmd(line: &str, mode: &str, inject: bool) -> (Option<IpcCommand>, Reply) {
        init_aliases();
        handle_command(line, mode, inject)
    }

    struct SocketDir(PathBuf);

    impl SocketDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("xremap-socket-{}-{:x}", std::process::id(), fastrand::u64(..)));
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
            .map(|_| UnixStream::connect(&path).unwrap()).collect();
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
        let mut batch = server.handle_readable(fd).or_else(|| server.handle_readable(fd)).unwrap();
        assert_eq!(batch.lines, [Err("line too long".into())]);
        batch.reply(&Reply::Err("line too long".into())).unwrap();
        drop(batch);
        let mut reply = String::new();
        client.read_to_string(&mut reply).unwrap();
        assert_eq!(reply, "err line too long\n");
        assert!(!server.holds_fd(fd));
    }


    #[test]
    fn ping() {
        assert_eq!(cmd("ping", "insert", false), (None, Reply::Pong));
    }

    #[test]
    fn get_mode() {
        assert_eq!(cmd("get-mode", "normal", false), (None, Reply::Mode("normal".into())));
    }

    #[test]
    fn set_mode() {
        assert_eq!(
            cmd("set-mode normal", "insert", false),
            (Some(IpcCommand::SetMode("normal".into())), Reply::Ok)
        );
    }

    #[test]
    fn set_mode_trims_whitespace() {
        assert_eq!(
            cmd("  set-mode   visual  \n", "insert", false),
            (Some(IpcCommand::SetMode("visual".into())), Reply::Ok)
        );
    }

    #[test]
    fn set_mode_rejects_bad_names() {
        for arg in ["", "with space", " Semi；colon", &"x".repeat(MAX_MODE_LEN + 1)] {
            let line = format!("set-mode {arg}");
            assert_eq!(
                cmd(&line, "insert", false).1,
                Reply::Err("invalid mode name".into()),
                "{arg:?}"
            );
        }
    }

    #[test]
    fn reload_config() {
        assert_eq!(
            cmd("reload-config", "insert", false),
            (Some(IpcCommand::ReloadConfig), Reply::Ok)
        );
    }

    #[test]
    fn unknown_and_empty() {
        assert_eq!(cmd("explode", "insert", false).1, Reply::Err("unknown command".into()));
        assert_eq!(cmd("", "insert", false).1, Reply::Err("empty command".into()));
    }

    #[test]
    fn key_disabled_by_default() {
        assert_eq!(
            cmd("key raw f13", "insert", false).1,
            Reply::Err("injection disabled".into())
        );
    }

    #[test]
    fn key_single_chord_defaults_to_raw() {
        assert_eq!(
            cmd("key f13", "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: None,
                    chords: vec![Chord { modifiers: vec![], key: Key::KEY_F13 }],
                    phases: KeyPhases::Tap,
                }),
                Reply::Ok
            )
        );
    }

    #[test]
    fn key_scope_and_multiple_chords() {
        assert_eq!(
            cmd("key expmap ctrl+shift+a f13", "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Expmap,
                    device: None,
                    chords: vec![
                        Chord { modifiers: vec![Key::KEY_LEFTCTRL, Key::KEY_LEFTSHIFT], key: Key::KEY_A },
                        Chord { modifiers: vec![], key: Key::KEY_F13 }
                    ],
                    phases: KeyPhases::Tap,
                }),
                Reply::Ok
            )
        );
    }

    #[test]
    fn key_device_option() {
        // Before and after the scope keyword.
        for line in ["key raw --device my-kbd f13", "key --device my-kbd raw f13"] {
            assert_eq!(
                cmd(line, "insert", true),
                (
                    Some(IpcCommand::SendKey {
                        scope: InjectScope::Raw,
                        device: Some("my-kbd".into()),
                        chords: vec![Chord { modifiers: vec![], key: Key::KEY_F13 }],
                        phases: KeyPhases::Tap,
                    }),
                    Reply::Ok
                ),
                "{line}"
            );
        }
        assert_eq!(
            cmd("key raw --device", "insert", true).1,
            Reply::Err("--device requires a name".into())
        );
    }

    #[test]
    fn key_repeat_suffix() {
        assert_eq!(
            cmd("key raw Down*3", "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: None,
                    chords: vec![
                        Chord { modifiers: vec![], key: Key::KEY_DOWN },
                        Chord { modifiers: vec![], key: Key::KEY_DOWN },
                        Chord { modifiers: vec![], key: Key::KEY_DOWN },
                    ],
                    phases: KeyPhases::Tap,
                }),
                Reply::Ok
            )
        );
        assert_eq!(
            cmd("key raw ctrl+a*0", "insert", true).1,
            Reply::Err("repeat count must be 1..=16".into())
        );
        assert_eq!(
            cmd("key raw ctrl+a*x", "insert", true).1,
            Reply::Err("invalid repeat count: x".into())
        );
    }

    #[test]
    fn keydown_keyup_phases() {
        assert_eq!(
            cmd("keydown ctrl+a", "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: None,
                    chords: vec![Chord { modifiers: vec![Key::KEY_LEFTCTRL], key: Key::KEY_A }],
                    phases: KeyPhases::Down,
                }),
                Reply::Ok
            )
        );
        assert_eq!(
            cmd("keyup ctrl+a", "insert", true),
            (
                Some(IpcCommand::SendKey {
                    scope: InjectScope::Raw,
                    device: None,
                    chords: vec![Chord { modifiers: vec![Key::KEY_LEFTCTRL], key: Key::KEY_A }],
                    phases: KeyPhases::Up,
                }),
                Reply::Ok
            )
        );
    }

    #[test]
    fn click_command() {
        let (command, reply) = cmd("click left right", "insert", true);
        assert_eq!(reply, Reply::Ok);
        assert_eq!(
            command,
            Some(IpcCommand::SendKey {
                scope: InjectScope::Raw,
                device: None,
                chords: vec![
                    Chord { modifiers: vec![], key: Key::BTN_LEFT },
                    Chord { modifiers: vec![], key: Key::BTN_RIGHT },
                ],
                phases: KeyPhases::Tap,
            })
        );
        assert_eq!(cmd("click 2", "insert", true).0, {
            Some(IpcCommand::SendKey {
                scope: InjectScope::Raw,
                device: None,
                chords: vec![Chord { modifiers: vec![], key: Key::BTN_MIDDLE }],
                phases: KeyPhases::Tap,
            })
        });
        assert_eq!(cmd("click", "insert", true).1, Reply::Err("click requires a button (left/middle/right)".into()));
        assert_eq!(
            cmd("click up", "insert", true).1,
            Reply::Err("unknown button: up (left/middle/right)".into())
        );
        assert_eq!(cmd("click left", "insert", false).1, Reply::Err("injection disabled".into()));
    }

    #[test]
    fn wheel_and_mousemove_commands() {
        assert_eq!(
            cmd("wheel -3", "insert", true),
            (Some(IpcCommand::Wheel { horizontal: false, amount: -3 }), Reply::Ok)
        );
        assert_eq!(
            cmd("hwheel 2", "insert", true),
            (Some(IpcCommand::Wheel { horizontal: true, amount: 2 }), Reply::Ok)
        );
        assert_eq!(cmd("wheel", "insert", true).1, Reply::Err("wheel takes an integer amount".into()));
        assert_eq!(cmd("wheel 1.5", "insert", true).1, Reply::Err("wheel takes an integer amount".into()));
        assert_eq!(
            cmd("mousemove 10 -5", "insert", true),
            (Some(IpcCommand::MoveMouse { x: 10, y: -5 }), Reply::Ok)
        );
        assert_eq!(cmd("mousemove 1", "insert", true).1, Reply::Err("mousemove takes X and Y coordinates".into()));
        assert_eq!(
            cmd("mousemove 1 2 3", "insert", true).1,
            Reply::Err("mousemove takes exactly two coordinates".into())
        );
        assert_eq!(cmd("mousemove a b", "insert", true).1, Reply::Err("invalid coordinate: a".into()));
        assert_eq!(cmd("wheel 1", "insert", false).1, Reply::Err("injection disabled".into()));
    }

    #[test]
    fn sleep_command() {
        assert_eq!(
            cmd("sleep 250", "insert", true),
            (Some(IpcCommand::Sleep(std::time::Duration::from_millis(250))), Reply::Ok)
        );
        // Does not require ipc.key; bounded.
        assert_eq!(
            cmd("sleep 0", "insert", true).1,
            Reply::Err("sleep takes 1..=5000 milliseconds".into())
        );
        assert_eq!(
            cmd("sleep 5001", "insert", true).1,
            Reply::Err("sleep takes 1..=5000 milliseconds".into())
        );
        assert_eq!(
            cmd("sleep abc", "insert", true).1,
            Reply::Err("sleep takes 1..=5000 milliseconds".into())
        );
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
    fn key_all_scopes() {
        for word in ["raw", "modmap", "keymap", "expmap", "experimental_map"] {
            let line = format!("key {word} x");
            let (command, _) = cmd(&line, "insert", true);
            assert_eq!(command.unwrap().scope(), InjectScope::parse(word).unwrap(), "{word}");
        }
    }

    #[test]
    fn key_rejects_bad_chords() {
        for line in ["key", "key raw", "key raw foo+bar", "key raw ctrl+", "key warp f13"] {
            let (_, reply) = cmd(line, "insert", true);
            assert!(matches!(reply, Reply::Err(_)), "{line:?} → {reply:?}");
        }
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
        let conn_fd = server.fds().map(|fd| fd.as_raw_fd()).find(|fd| *fd != listener_fd).unwrap();

        client.write_all(b"set-mode normal\nget-mode\nping\n").unwrap();
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
