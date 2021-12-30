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
