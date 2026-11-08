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
  inject: false         # allow the key-injection commands (default: false)
```

| Option | Default | Meaning |
|---|---|---|
| `socket` | (none — IPC off) | `true` for the default path, or an explicit path. `false` explicitly disables. |
| `inject` | `false` | Gate for the whole injection command family (`key`, `keydown`, `keyup`, `click`, `wheel`, `hwheel`, `mousemove`, `sleep`). When off, the server answers `err injection disabled`. |

The default socket path falls back to `$TMPDIR/xremap-<uid>.sock` when
`$XDG_RUNTIME_DIR` is unset.

## Protocol

Line-based text, one command per line, one reply line per command — the same
shape as dotool's fifo protocol.

```
set-mode MODE\n                          → ok\n
get-mode\n                               → MODE\n
ping\n                                   → pong\n
key [--device NAME] [SCOPE] CHORD...\n   → ok\n
keydown [--device NAME] [SCOPE] CHORD...\n → ok\n
keyup [--device NAME] [SCOPE] CHORD...\n   → ok\n
click [left/middle/right]...        → ok\n
wheel AMOUNT\n                        → ok\n
hwheel AMOUNT\n                       → ok\n
mousemove X Y\n                      → ok\n
sleep MILLISECONDS\n                     → ok\n
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
- `CHORD` is `MODIFIER+...+KEY` where modifiers are
  `ctrl|control|alt|shift|super|meta|win|altgr` and the key name goes through
  the same parser as config keys (`f13`, `KEY_F13`, `CODE_191`, ...).
  Unlike dotool, a bare uppercase letter does not imply Shift:
  `key A` presses the `a` key without modifiers — write `key shift+a`.
  A chord may carry a `*N` repeat suffix (`Down*3` presses Down three times).
  Up to 16 chords per command, so `key raw f13 f14 ctrl+a*2` sends four
  keypresses in order.
- `--device NAME` (optional) sets the device identity of the injected events,
  i.e. which `device:` keymap conditions they match. It is a per-command
  property, chosen by the client. The default is `xremap-ipc`.
- `keydown`/`keyup` emit only the press / only the release half of each chord
  (`keydown ctrl+a` = ctrl↓ a↓, `keyup ctrl+a` = a↑ ctrl↑). Pair them
  yourself; held keys stay held.
- `click BUTTON...` presses and releases mouse buttons. Buttons are `left`,
  `middle` and `right` (also `1`/`2`/`3`); each maps to the corresponding
  evdev `BTN_*` key, i.e. `click left` is shorthand for `key raw btn_left`.
- `wheel AMOUNT` and `hwheel AMOUNT` scroll vertically (REL_WHEEL) or
  horizontally (REL_HWHEEL) by AMOUNT lines/columns. Positive amounts scroll
  up/right, negative down/left, following the Linux evdev convention. AMOUNT
  is an integer. Wheel events require `enable_wheel: true` (the default).
  When that option is disabled the output device has no wheel axis and the
  command returns `err wheel events are disabled (...)` instead of reporting a
  scroll that never happens.
- `mousemove X Y` moves the pointer by X/Y units (REL_X / REL_Y), dispatched
  as one batch. Both are integers; there is no absolute positioning, which
  would require a second, absolute virtual device.
- `sleep` pauses command processing for 1..=5000 ms, so a sequence can space
  out keys for applications that need delays: `xremap sleep 100`.
  Note that it blocks xremap's event loop: no physical keys are remapped
  while it sleeps, so keep the duration short.

## The `key` scopes

`SCOPE` is the pipeline layer the injected key **enters** at; from there it
flows on to the application. The pipeline is **experimental_map (expmap)
operators → modmap → keymap → app**. Each scope name is its entering layer.

| Scope | Flow | Meaning |
|---|---|---|
| `raw` (default) | → app | Emit straight to the output device. No rule can rewrite it. Same semantics as dotool. |
| `keymap` | → keymap → app | Triggers keymap rules (mode/application/window/device conditions, nested remaps); modmap is skipped. |
| `modmap` | → modmap → keymap → app | Rewritten by `modmap` first, then flows through keymap. experimental_map operators are skipped. |
| `expmap` | → expmap → modmap → keymap → app | Enters at the experimental_map operators: indistinguishable from a physical keypress; operators may buffer or reorder it. `experimental_map` is accepted as an alias. |

Injected events carry a synthetic device identity (`--device NAME`, default
`xremap-ipc`), so you can write keymap entries that only apply to injected
keys:

```yaml
keymap:
  - name: rules only for injected keys
    device:
      only: [xremap-ipc]
    remap:
      f13: { set_mode: normal }
```

The injection sequence per chord is: modifier presses → key press → key
release → modifier releases (reverse order), and it is dispatched through the
same output device and throttle as remapped keys.

## Security

- The socket is created with mode 0600 inside `$XDG_RUNTIME_DIR` (mode 0700,
  per-user tmpfs) by default, so only the owning user can connect. On Linux,
  connections are additionally checked with `SO_PEERCRED`: a root peer is
  always accepted, otherwise the peer must run as the same uid as xremap.
  Note that when xremap itself runs as root the uid comparison admits any
  peer; access control then rests on the 0600 socket mode and its parent
  directory, not on peer credentials.
- The command set is a fixed whitelist; clients cannot supply arbitrary
  executable names or arguments. Pipeline key injection can trigger configured
  actions under the normal `allow_launch` policy.
- At most 64 connections are admitted. Accept resource errors back off for
  500 ms instead of stopping the daemon or spinning in the input loop.
- Startup only removes stale sockets owned by xremap's effective uid. Live
  sockets, ordinary files, symlinks, and foreign sockets are not replaced.
- The injection command family (`key`/`keydown`/`keyup`/`click`/`wheel`/
  `hwheel`/`mousemove`/`sleep`) is off by default
  (`ipc.inject: false`) and only widens what a same-uid process can already do
  (e.g. via dotool/uinput). It does not grant cross-user power.

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
$ xremap key expmap ctrl+f
ok
$ xremap key --device my-kbd raw f13
ok
```

Exit code is non-zero if the command fails. Override the socket with
`--socket PATH` (anywhere on the command line before a literal `--`
argument separator) or the `XREMAP_SOCKET` environment variable.

### Any line-oriented socket tool

The protocol needs no framing, so debugging is trivial:

```console
$ socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/xremap.sock
ping
pong
```

## Status

Experimental: the protocol may evolve before it is declared stable.
The verb namespace is open: new injection verbs join the existing `inject`
gate.
