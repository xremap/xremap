# IPC control socket

xremap can expose a unix domain socket that lets other programs query and
control a running instance: switching keymap modes, reloading the config, and
(optionally) injecting key events. This is how external modal systems (e.g. an
Emacs modal editing package) keep xremap's `mode` in sync without resorting to
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
reload-config\n                          → ok\n
ping\n                                   → pong\n
key [--device NAME] [SCOPE] CHORD...\n   → ok\n
keydown [--device NAME] [SCOPE] CHORD...\n → ok\n
keyup [--device NAME] [SCOPE] CHORD...\n   → ok\n
type [--layout LAYOUT] [--variant VARIANT] [--base64 PAYLOAD] [--] TEXT\n → ok\n
click [left/middle/right]...        → ok\n
wheel AMOUNT\n                        → ok\n
hwheel AMOUNT\n                       → ok\n
mousemove X Y\n                      → ok\n
sleep MILLISECONDS\n                     → ok\n
```

Commands on a connection execute in order, and each reply follows execution.
For example, `set-mode normal\nget-mode\n` returns `ok\nnormal\n`,
including when both lines arrive in one write. Execution failures return `err`
without terminating the daemon; later commands can still run. Configured exit
or full-restart key actions intentionally stop further commands on the connection.
Replies confirm
xremap processing, not that the focused application has consumed its input.

Errors are `err REASON\n`. Free-form reply text escapes backslashes, CR and LF
as `\\`, `\r` and `\n` so every reply stays on one line; this applies to
`REASON` and to `MODE`. Input must be valid UTF-8. Lines longer than 4096 bytes
are rejected and the connection is closed.


- `MODE` is an opaque string matched against `mode:` keymap conditions
  (`[a-zA-Z0-9_-]`, max 64 bytes for `set-mode`). It switches the mode exactly
  like the `{ set_mode: ... }` keymap action. Config/keymap modes may contain
  other characters; the `get-mode` reply is the bare escaped mode name and
  stays on one line. Clients tell replies apart by order, and a reply is a
  failure exactly when it starts with `err`.
- `reload-config` reloads the config files without touching devices. Runtime
  mode and connections are preserved when the socket path stays the same.
  Failed config loads or socket binds return `err` and retain the old config,
  including its key aliases.
  A path change replies on the old connection before it closes; reconnect to
  the new path. Removing `ipc` disables further commands.
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
- `type` types `TEXT`. The rest of the line is the text; `\\n`, `\\t` and `\\\\`
  escapes let newlines and tabs travel over the line-based protocol. A bare
  newline cannot appear in `TEXT`: it terminates the command, and letting it
  through would turn typed text into commands. `--base64 PAYLOAD` takes the
  same text encoded as base64 instead (see [Typing large text](#typing-large-text)).
  `--layout`/`--variant` select the XKB layout, defaulting to
  `$XKB_DEFAULT_LAYOUT` and then `us`. The layout must be the one the
  receiving application decodes with — normally the session layout, not "the
  layout of the keycaps": injection sends keycodes, so `--layout de` on a `us`
  session types `ü` as `[` and `ß` as `-`. Layout resolution uses libxkbcommon,
  loaded at runtime — no build-time dependency.

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

  ```console
  $ xremap --client 'type --layout de Grüße'
  ```

  Options are `--token` style and must come first. A misspelt option is an

  Limits for `type`: `TEXT` is at most 4096 characters, a protocol line at
  most 16384 bytes, and a `--base64` payload at most 8192 bytes once decoded.
- `sleep` pauses command processing for 1..=5000 ms, so a command stream can
  sequence keys with delays for applications that need them:
  `printf 'key raw alt+f\nsleep 100\nkey raw down\n' | xremap --client`.
  Note that it blocks xremap's event loop: no physical keys are remapped
  while it sleeps, so keep the duration short.

## Typing large text

Escaping is fine for a sentence, not for a paragraph. `TEXT` must stay on one
line and is capped, so multi-line or long text should not be assembled by hand:

```console
$ cat notes.txt | xremap --client 'type --stdin'
$ cat notes.txt | xremap --client 'type --stdin --layout de --restore'
$ xremap --client 'type --file notes.txt'
$ printf 'a\r\nb\r\n' | xremap --client 'type --stdin'   # CRLF is normalised
```

`--stdin` and `--file PATH` are expanded by the **client**: it reads the text,
normalises CRLF and lone CR to LF (xremap can inject Enter but has no CR
key), base64-encodes it, and sends a single `type --base64 PAYLOAD` command.
Consequences:

- Nothing is chunked or re-sent per line: multi-line text is one command, so
  order and newlines come from the text itself, not from extra `key` commands.
- No escaping is involved anywhere: the payload is base64, which cannot
  contain a line break.
- The server never reads files or stdin; `--file` is client-side only.
- `--stdin` cannot be combined with `xremap --client -`, because stdin already
  carries commands there.
- Inline text cannot be combined with `--stdin`/`--file`.

`--base64` is also usable directly by any client that already has the text
encoded; a bad payload returns `err invalid base64: ...`, non-UTF-8 bytes
return `err base64 payload is not valid UTF-8`, and oversized text returns
`err text too long (...)`.

### Why not raw multi-line text?

The protocol is line-based, so a bare newline is a frame boundary. Accepting it
inside `TEXT` would let typed text be executed as commands — for example a
pasted line `key raw ctrl+alt+delete` would press those keys. dotool has the
same constraint and simply cannot type a newline at all; xremap can, via `\\n`
or `--base64`.

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
  executable names or arguments. Pipeline key injection can trigger configured actions under the normal
  `allow_launch` policy.
- At most 64 connections are admitted. Accept resource errors back off for
  500 ms instead of stopping the daemon or spinning in the input loop.
- Startup only removes stale sockets owned by xremap's effective uid. Live
  sockets, ordinary files, symlinks, and foreign sockets are not replaced.
- The injection command family (`key`/`keydown`/`keyup`/`type`/`click`/
  `wheel`/`hwheel`/`mousemove`/`sleep`) is off by default (`ipc.inject: false`)
  and only widens what a same-uid process can already do (e.g. via
  dotool/uinput). It does not grant cross-user power.

## Clients

### Built-in client mode

```console
$ xremap --client 'set-mode normal'
ok
$ echo 'key expmap ctrl+f' | xremap --client -
ok
$ xremap --client get-mode
normal
```

Quote commands containing spaces so the entire command is passed as one
argument. `xremap --client -` reads one command per line from stdin over a single
connection. Exit code is non-zero if any command fails. Override the socket
with `--socket PATH` or the `XREMAP_SOCKET` environment variable.

### Any line-oriented socket tool

The protocol needs no framing, so debugging is trivial:

```console
$ socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/xremap.sock
ping
pong
```

### Emacs (no external process)

```elisp
(defun my/xremap-ipc (command)
  (condition-case err
      (let ((proc (make-network-process
                   :name "xremap-ipc" :family 'local :noquery t
                   :service (or (getenv "XREMAP_SOCKET")
                                (expand-file-name
                                 "xremap.sock"
                                 (or (getenv "XDG_RUNTIME_DIR") "/run/user/1000"))))))
        (process-send-string proc (concat command "\n"))
        (delete-process proc))
    (error (message "xremap ipc: %s" err))))

;; keep xremap's mode in sync with a modal editing state machine
(defun my/ewm-sync-xremap-mode ()
  (my/xremap-ipc (concat "set-mode " (symbol-name my-modal-state))))
(add-hook 'my-modal-state-change-hook #'my/ewm-sync-xremap-mode)
```

## Status

Experimental: the protocol may evolve before it is declared stable.
The verb namespace is open: new injection verbs join the existing `inject`
gate.
