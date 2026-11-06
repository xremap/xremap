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
| `inject` | `false` | Gate for the whole injection command family (`key`, `keydown`, `keyup`, `type`, `click`, `wheel`, `hwheel`, `mousemove`, `sleep`). When off, the server answers `err injection disabled`. |

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
type [OPTIONS] TEXT\n                    → ok\n  (--method auto|layout|unicode|clipboard, --layout, --variant,\n   --paste CHORD, --restore, --base64 PAYLOAD)\n
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
`REASON` and to `MODE`. Input must be valid UTF-8. Lines longer than 16384 bytes
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
  out keys for applications that need delays: `xremap sleep 100`. It blocks
  xremap's event loop, so no physical keys are remapped while it sleeps;
  keep the duration short.
- `type` types `TEXT`. The rest of the line is the text; `\n`, `\t` and `\\`
  escapes let newlines and tabs travel over the line-based protocol. A bare
  newline cannot appear in `TEXT`: it terminates the command, and letting it
  through would turn typed text into commands. `--base64 PAYLOAD` takes the
  same text encoded as base64 instead (see [Typing large text](#typing-large-text)).
  `--layout`/`--variant` select the XKB layout, defaulting to
  `$XKB_DEFAULT_LAYOUT` and then `us`. The layout must be the one the
  receiving application decodes with — normally the session layout, not "the
  layout of the keycaps": injection sends keycodes, so `--layout de` on a `us`
  session types `ü` as `[` and `ß` as `-`. Layout resolution uses libxkbcommon,
  linked into the binary at build time (statically for the musl release
  binaries; otherwise dynamically).

  Because injection sends keycodes, a locked CapsLock in the session flips the
  case of the letters: `type hello` types `HELLO`. The CapsLock state is not
  observable on the injection path.

  Two- and three-key dead-key sequences are resolved through libxkbcommon's
  Compose table (`$XCOMPOSEFILE`, user XCompose files, or the locale table).
  For example, a matching layout can produce `é` via `dead_acute` then `e`.
  The receiving application must use compatible composition rules. The keys
  involved must exist on the layout; otherwise the method's fallback applies.
  Custom Compose results containing several characters or exceeding the result
  buffer are not registered as single-character mappings.

  `--method` picks how characters are produced:

  | `--method` | Behaviour |
  |---|---|
  | `auto` (default) | Use a real key of the layout; anything the layout cannot produce falls back to `Ctrl+Shift+U`, the hex codepoint and Enter. |
  | `layout` | Use only real keys. Characters the layout cannot produce are an error naming the character. |
  | `unicode` | Always use the `Ctrl+Shift+U` sequence. |
  | `clipboard` | Put the text on the clipboard and send a paste keystroke. |

  `Ctrl+Shift+U` is an application/input-method convention, not a universal
  X11/XKB input mechanism. It works in supporting environments (for example,
  some GTK input contexts); support cannot be inferred just from Qt, Chromium,
  Electron, or terminal use. In unsupported applications it may type literal
  hex digits or trigger shortcuts. Use `--method layout` for strict layout-only
  typing, or `--method clipboard` when the target supports pasting.

  `--method unicode` and `--method clipboard` do not depend on the layout's
  keycode levels, so they also bypass the CapsLock case flip described above.

  ```console
  $ xremap type --layout de Grüße
  $ xremap type --method clipboard 中文
  $ xremap type --method clipboard --paste ctrl+shift+v 中文   # terminals
  $ xremap type --method clipboard --paste ctrl+y 中文         # Emacs
  $ xremap type --method clipboard --restore --paste ctrl+shift+v 中文
  ```

  Options are `--token` style and must come first. A misspelt option is an
  error (`unknown option '---paste'`) rather than being typed out as text;
  write `type -- TEXT` to type text that itself starts with a dash. `--restore`
  may appear anywhere among the options.

  Limits for `type`: `TEXT` is at most 4096 characters, a protocol line at
  most 16384 bytes, and a `--base64` payload at most 8192 bytes once decoded.

  `--paste` is the keystroke that pastes, `modifier+key` syntax, default
  `ctrl+v`. `--restore` best-effort restores the previous text clipboard 500 ms
  after dispatching the paste. This snapshots at most 16 MiB of UTF-8 text, not
  images or other MIME types. Snapshot failure leaves the newly written text.
  The delay is not a paste acknowledgment: a late reader can get the restored
  text, and intervening user clipboard changes or concurrent paste commands
  can race with restoration. Without this flag the clipboard keeps the text.

  Clipboard mode runs `wl-copy`, `xsel` or `xclip` with fixed arguments, so it
  is independent of `allow_launch`. It synchronously blocks the input event
  loop while accessing the clipboard: reads have a 200 ms deadline per helper,
  writes a 1 s deadline per helper, followed by 50 ms settling on success.
  Failed helpers may be tried sequentially. Ordinary helper failures return
  `err` rather than terminating xremap.

## Typing large text

Escaping is fine for a sentence, not for a paragraph. `TEXT` must stay on one
line and is capped, so multi-line or long text should not be assembled by hand:

```console
$ xremap type --stdin < notes.txt
$ xremap type --stdin --layout de < notes.txt
$ printf 'a\r\nb\r\n' | xremap type --stdin   # CRLF is normalised
```

`--stdin` is expanded by the **client**: it reads the text,
normalises CRLF and lone CR to LF (xremap can inject Enter but has no CR
key), base64-encodes it, and sends a single `type --base64 PAYLOAD` command.
Consequences:

- Nothing is chunked or re-sent per line: multi-line text is one command, so
  order and newlines come from the text itself, not from extra `key` commands.
- No escaping is involved anywhere: the payload is base64, which cannot
  contain a line break.
- Inline text cannot be combined with `--stdin`.

`--base64` is also usable directly by any client that already has the text
encoded; a bad payload returns `err invalid base64: ...`, non-UTF-8 bytes
return `err base64 payload is not valid UTF-8`, and oversized text returns
`err text too long (...)`.

### Why not raw multi-line text?

The protocol is line-based, so a bare newline is a frame boundary. Accepting it
inside `TEXT` would let typed text be executed as commands — for example a
pasted line `key raw ctrl+alt+delete` would press those keys. dotool has the
same constraint and simply cannot type a newline at all; xremap can, via `\n`
or `--base64`.

The built-in client rejects bare CR and LF in command arguments before
connecting. Use escaped text, `--stdin` or `--base64` for line breaks. Use
`xremap type -- TEXT` when literal text starts with a dash.

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
  executable names or arguments. Clipboard operations use fixed helpers, and
  pipeline key injection can trigger configured actions under the normal
  `allow_launch` policy.
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
