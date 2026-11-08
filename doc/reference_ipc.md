# IPC control socket

xremap can expose a unix domain socket that lets other programs query and
control a running instance. This is how external modal systems (e.g. an Emacs
modal editing package) keep xremap's state in sync without resorting to
synthetic keyboards like dotool/ydotool.

The feature is **disabled by default**.

## Configuration

```yaml
ipc:                    # omit the whole section to disable IPC entirely
  socket: true          # listen on $XDG_RUNTIME_DIR/xremap.sock (or an explicit path)
  # socket: /run/user/1000/my-xremap.sock
```

| Option | Default | Meaning |
|---|---|---|
| `socket` | (none — IPC off) | `true` for the default path, or an explicit path. `false` explicitly disables. |

The default socket path falls back to `$TMPDIR/xremap-<uid>.sock` when
`$XDG_RUNTIME_DIR` is unset.

## Protocol

Line-based text, one command per line, one reply line per command — the same
shape as dotool's fifo protocol.

```
set-mode MODE\n                          → ok\n
get-mode\n                               → MODE\n
ping\n                                   → pong\n
```

Commands on a connection execute in order, and each reply follows execution.
Execution failures return `err` without terminating the daemon; later commands
can still run.

Errors are `err REASON\n`. Free-form reply text escapes backslashes, CR and LF
as `\\`, `\r` and `\n` so every reply stays on one line; this applies to
`REASON` and to `MODE`. Input must be valid UTF-8. Lines longer than 256 bytes
are rejected and the connection is closed.

- `MODE` is an opaque string matched against `mode:` keymap conditions
  (`[a-zA-Z0-9_-]`, max 64 bytes for `set-mode`). It switches the mode exactly
  like the `{ set_mode: ... }` keymap action. Config/keymap modes may contain
  other characters; the `get-mode` reply is the bare escaped mode name and
  stays on one line. Clients tell replies apart by order, and a reply is a
  failure exactly when it starts with `err`.

## Security

- The socket is created with mode 0600 inside `$XDG_RUNTIME_DIR` (mode 0700,
  per-user tmpfs) by default, so only the owning user can connect. On Linux,
  connections are additionally checked with `SO_PEERCRED`: a root peer is
  always accepted, otherwise the peer must run as the same uid as xremap.
  Note that when xremap itself runs as root the uid comparison admits any
  peer; access control then rests on the 0600 socket mode and its parent
  directory, not on peer credentials.
- The command set is a fixed whitelist; clients cannot supply arbitrary
  executable names or arguments.
- At most 64 connections are admitted. Accept resource errors back off for
  500 ms instead of stopping the daemon or spinning in the input loop.
- Startup only removes stale sockets owned by xremap's effective uid. Live
  sockets, ordinary files, symlinks, and foreign sockets are not replaced.

## Clients

### Built-in client mode

The first argument selects a subcommand; everything else keeps its usual
meaning (config files). Each invocation sends one command:

```console
$ xremap set-mode normal
ok
$ xremap get-mode
normal
$ xremap ping
pong
```

Exit code is non-zero if the command fails. Override the socket with
`--socket PATH` (anywhere on the command line) or the `XREMAP_SOCKET`
environment variable.

### Any line-oriented socket tool

The protocol needs no framing, so debugging is trivial:

```console
$ socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/xremap.sock
ping
pong
```

## Status

Experimental: the protocol may evolve before it is declared stable.
