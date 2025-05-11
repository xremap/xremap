## v0.10.12

- Add `{ press: }`, `{ repeat: }`, and `{ release: }` keymap actions [#678](https://github.com/xremap/xremap/pull/678)
- Add `repeat` field for `press`/`release` modmap [#678](https://github.com/xremap/xremap/pull/678)

## v0.10.11

- Add mode for modmap
  [#675](https://github.com/xremap/xremap/pull/675)

## v0.10.10

- Support deleting and re-adding a config with `--watch=config`
  [#673](https://github.com/xremap/xremap/pull/673)

## v0.10.9

- Use an appropriate parser for secondary TOML files [#665](https://github.com/xremap/xremap/pull/665)
- Let `--watch=config` reload the config even when the timestamp is not changed [#666](https://github.com/xremap/xremap/pull/666)

## v0.10.8

- Allow matching event device on udev properties with `--features udev` build
  [#587](https://github.com/xremap/xremap/pull/587)

## v0.10.7

- Enable LTO in release builds [#643](https://github.com/xremap/xremap/pull/643)

## v0.10.6

- Add `--vendor` and `--product` options to specify xremap's uinput product / vendor IDs
  [#583](https://github.com/xremap/xremap/pull/583)

## v0.10.5

- Allow matching any key with `ANY` [#575](https://github.com/xremap/xremap/pull/575)
- Ignore absolute input devices on `--mouse` [#577](https://github.com/xremap/xremap/pull/577)
- Detect keyboards with a mouse button by default [#550](https://github.com/xremap/xremap/pull/550)

## v0.10.4

- Add sleep KeymapAction [#574](https://github.com/xremap/xremap/pull/574)

## v0.10.3

- Allow keymap with no action [#544](https://github.com/xremap/xremap/pull/544)
- hypr: Support window title match

## v0.10.2

- Resurrect `hypr` client that was dropped in v0.10.0
  - Hyprland stopped using wlroots ([ref](https://hyprland.org/news/independentHyprland/)),
    so `wlroots` client no longer works for it.

## v0.10.1

- X11: Handle WM\_CLASS reply without terminating null-byte [#525](https://github.com/xremap/xremap/pull/525)

## v0.10.0

- Drop `sway` and `hypr` clients [#479](https://github.com/xremap/xremap/pull/479)
  - As recommended since v0.8.9, please use `wlroots` client instead.

## v0.9.0

- Add `enable_wheel` option in the config [#478](https://github.com/xremap/xremap/pull/478)
- Enable `REL_WHEEL` and `REL_HWHEEL` by default regardless of `--mouse` option.
  [#478](https://github.com/xremap/xremap/pull/478)
  - This reverts v0.8.2 and v0.8.10.
  - Arch Linux users on systemd-253-1 can continue to disable them by top-level `enable_wheel: false`.
    See [#260](https://github.com/xremap/xremap/pull/260) for details.
- Emit no key event instead of `KEY_UNKNOWN` on `skip_key_event` [#462](https://github.com/xremap/xremap/pull/462)

## v0.8.18

- Fix issues in the release pipeline

## v0.8.17

- Add `window` matcher for the `kde` client [#448](https://github.com/xremap/xremap/pull/448)

## v0.8.16

- Add `window` matcher for the `wlroots` client [#447](https://github.com/k0kubun/xremap/issues/447)
- Handle new KWin API for the `kde` client [#437](https://github.com/k0kubun/xremap/issues/437)

## v0.8.15

- Add `skip_key_event` option to `press`/`release` modmap action [#420](https://github.com/k0kubun/xremap/issues/420)

## v0.8.14

- Support TOML as a config file format [#404](https://github.com/k0kubun/xremap/issues/404)

## v0.8.13

- Add `shared` field for anchors and aliases [#402](https://github.com/k0kubun/xremap/issues/402)

## v0.8.12

- Add `device` filter to `modmap` and `keymap` entries [#380](https://github.com/k0kubun/xremap/issues/380)

## v0.8.11

- Use `xremap` instead of `xremap pid=$pid` as the device name if it doesn't conflict
  - If there's already an `xremap` device, it uses `xremap pid=$pid` as before

## v0.8.10

- Add `REL_WHEEL` and `REL_HWHEEL` to relative axes if `--mouse` is given
  - This resurrects what's dropped in v0.8.2 if you use that option

## v0.8.9

- Introduce `wlroots` support [#345](https://github.com/k0kubun/xremap/issues/345)
  - `sway` and `hypr` users are encourated to switch to this

## v0.8.8

- Let `keypress_delay_ms` delay key releases as well [#341](https://github.com/k0kubun/xremap/issues/341)

## v0.8.7

- Support multi-key `held`/`alone` in `modmap` [#339](https://github.com/k0kubun/xremap/issues/339)

## v0.8.6

- Update clap from v3 to v4
  - `--help` no longer has colors

## v0.8.5

- Update Hyperland-rs from 0.3.0 to 0.3.1 [#275](https://github.com/k0kubun/xremap/issues/275)

## v0.8.4

- Support loading multiple configs [#269](https://github.com/k0kubun/xremap/issues/269)

## v0.8.3

- Support KDE Wayland [#264](https://github.com/k0kubun/xremap/issues/264)

## v0.8.2

- Remove `REL_WHEEL` and `REL_HWHEEL` as a workaround for Arch Linux with systemd-253-1

## v0.8.1

- Update Hyperland-rs from 0.2.4 to 0.3.0 [#247](https://github.com/k0kubun/xremap/issues/247)

## v0.8.0

- Support scrollwheel / RELATIVE events
  [#180](https://github.com/k0kubun/xremap/issues/180)
  [#187](https://github.com/k0kubun/xremap/issues/187)

## v0.7.15

- Handle a None application name on Hyperland [#220](https://github.com/k0kubun/xremap/issues/220)

## v0.7.14

- Fix overrides with multi-key sequences [#217](https://github.com/k0kubun/xremap/issues/217)

## v0.7.13

- Support Hyperland [#216](https://github.com/k0kubun/xremap/issues/216)

## v0.7.12

- Double-fork a `launch`ed process to avoid stopping it when xremap exits [#214](https://github.com/k0kubun/xremap/issues/214)

## v0.7.11

- Reconnect to X11 when an existing connection stops working [#212](https://github.com/k0kubun/xremap/issues/212)

## v0.7.10

- Introduce `keymap`-level `exact_match` option [#209](https://github.com/k0kubun/xremap/issues/209)

## v0.7.9

- Press keymap modifiers before releasing modifiers not in keymap [#208](https://github.com/k0kubun/xremap/issues/208)

## v0.7.8

- Fix libinput disable-while-typing support [#152](https://github.com/k0kubun/xremap/issues/152)

## v0.7.7

- Flush pending keys with an override remap correctly [#154](https://github.com/k0kubun/xremap/issues/154)

## v0.7.6

- Add `keypress_delay_ms` to workaround [#179](https://github.com/k0kubun/xremap/issues/179)
  - This feature might be removed in the future once the root cause of [#179](https://github.com/k0kubun/xremap/issues/179)
    is found and fixed.

## v0.7.5

- Fix a regression to recognize JetBrains IDE on `application` since v0.5.3
  [#151](https://github.com/k0kubun/xremap/issues/151)

## v0.7.4

- Fix nested remap implementation that has been broken since v0.6.0
  [#149](https://github.com/k0kubun/xremap/issues/149)

## v0.7.3

- Dispatch the original key as well on v0.5.1's `press`/`release` modmap

## v0.7.2

- Fix a corner-case bug in the v0.7.1 feature

## v0.7.1

- Keep extra modifiers when a remapped combination is pressed
  [#102](https://github.com/k0kubun/xremap/issues/102)

## v0.7.0

- Introduce `virtual_modifiers` [#147](https://github.com/k0kubun/xremap/pull/147)
- Obsolete the `modifier: true` feature of v0.6.1 in favor of `virtual_modifiers`

## v0.6.2

- Avoid emitting logical modifiers when a key combination is pressed

## v0.6.1

- Support logical modifier keys by `modifier: true` in `modmap`
  [#146](https://github.com/k0kubun/xremap/pull/146)

## v0.6.0

- Rewrite the modifier key match logic
  - Match remaps even if extra modifiers are held [#102](https://github.com/k0kubun/xremap/issues/102)
  - Even faster than the previous version

## v0.5.3

- Match an `application` matcher against a full `WM_CLASS` in X11
  when the matcher contains `.`.
  - If you don't specify `.`, it's backward-compatible.
  - If you already use v0.4.5's `/regex/`, however, you might need to tweak the regex.

## v0.5.2

- Support overriding `timeout_key` on nested remap
  [#144](https://github.com/k0kubun/xremap/pull/144)

## v0.5.1

- Support triggering `keymap` actions on key press/release events
  [#79](https://github.com/k0kubun/xremap/pull/79)

## v0.5.0

- Switch to binary releases built by cross-rs
  - Support Arm64
  - All binaries no longer have dynamic-link dependencies like libc
  - Stop supporting "others" distribution
     - If you use none of X11, GNOME, or Sway, try X11. It might help XWayland.

## v0.4.6

- Add `--mouse` option to select mouse as well
  [#140](https://github.com/k0kubun/xremap/pull/138)
- The X11 binary no longer relies on libx11

## v0.4.5

- `application` supports a regular expression matcher, `/regex/`
  [#138](https://github.com/k0kubun/xremap/pull/138)

## v0.4.4

* Support Vim-like modal remapping by `mode`, `set_mode`, and `default_mode`
  [#93](https://github.com/k0kubun/xremap/pull/93)

## v0.4.3

* Produce xremap binary releases on Ubuntu 18.04
  [#101](https://github.com/k0kubun/xremap/pull/101)

## v0.4.2

* `--features gnome`: Support a new protocol to talk to the GNOME Shell extension
  * Both old and new GNOME Shell extensions work with this version

## v0.4.1

* `--features gnome`: Show `supported: false` in the command output
  when the GNOME Shell extension is not installed

## v0.4.0

* `--features gnome`: Depend on an external GNOME Shell extension, [xremap-gnome](https://github.com/xremap/xremap-gnome)
  * If you use `--features gnome`, install [xremap's GNOME Shell extension](https://extensions.gnome.org/extension/5060/xremap/)
  * This allows you to use xremap with GNOME 40+. Ubuntu 22.04 uses GNOME 42 for example.

## v0.3.3

* Fix a bug in a nested remap with modifiers
  [#91](https://github.com/k0kubun/xremap/pull/91)

## v0.3.2

* Support `timeout_millis` to timeout a prefix key
  [#82](https://github.com/k0kubun/xremap/pull/82)

## v0.3.1

* Keep `--watch` working when multiple keyboards are used
  [#85](https://github.com/k0kubun/xremap/pull/85)

## v0.3.0

* Switch the CLI framework from getopts to clap
* Add `--watch=config` to watch configs
  * `--watch` works as before. You may use it as `--watch=device` as well.
* Add `--completions` for shell completion

## v0.2.5

* Support `escape_next_key` action
  [#74](https://github.com/k0kubun/xremap/pull/74)

## v0.2.4

* Use feature-related dependencies only when needed
  [#68](https://github.com/k0kubun/xremap/pull/68)

## v0.2.3

* Detect XWayland applications properly for Sway
  [#65](https://github.com/k0kubun/xremap/pull/65)

## v0.2.2

* Support `BTN_MISC`, `BTN_MOUSE`, `BTN_EXTRA`, `BTN_FORWARD`, `BTN_BACK`, and `BTN_TASK`
  as mouse buttons as well
  [#63](https://github.com/k0kubun/xremap/pull/63)

## v0.2.1

* Support remapping a mouse with `BTN_SIDE`
  [#57](https://github.com/k0kubun/xremap/pull/57)

## v0.2.0

* Support left/right-specific modifiers by `_L`/`_R` prefixes
  [#56](https://github.com/k0kubun/xremap/pull/56)

## v0.1.9

* Fix a bug of handling control keys inside `with_mark` of v0.1.7
  [#55](https://github.com/k0kubun/xremap/pull/55)

## v0.1.8

* Add `--version` option to show xremap's version
  [#54](https://github.com/k0kubun/xremap/issues/54)

## v0.1.7

* Add `set_mark` and `with_mark` to emulate Emacs's mark mode
  [#53](https://github.com/k0kubun/xremap/issues/53)

## v0.1.6

* Add `launch` action to execute a command
  [#52](https://github.com/k0kubun/xremap/issues/52)

## v0.1.5

* Add `--watch` option to automatically add new devices
* Avoid crashing on a disconnected device
* `name` is made optional in `modmap` and `keymap`

## v0.1.4

* Add `--ignore` option to deny-list devices instead of allow-listing them
  [#46](https://github.com/k0kubun/xremap/issues/46)
* Abort `xremap` when no device was selected

## v0.1.3

* Support remapping a key to two different keys depending on
  whether it's pressed alone or held
  [#47](https://github.com/k0kubun/xremap/issues/47)

## v0.1.2

* Fix recognition of a right Alt modifier in `keymap`
  [#43](https://github.com/k0kubun/xremap/issues/43)

## v0.1.1

* Binary distribution is built on GitHub Actions
* Improve error message for features `gnome` and `sway`
* Stop using a fork of swayipc and publish `sway` feature on crates.io

## v0.1.0

* Initial release
  * `modmap`, `keymap`, `application`, `remap`
  * --features: `x11`, `gnome`, `sway`
