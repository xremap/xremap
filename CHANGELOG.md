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
