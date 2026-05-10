## FreeBSD support

### Installation instructions

#### Install rust

```sh
# Should work, otherwise use https://sh.rustup.rs
pkg install rust
```

#### Compile xremap

```sh
cargo build --release --features x11
```

The compiled binary has the path `target/release/xremap`.

#### Set permissions

```sh
chmod -R g+rwx /dev/input
chmod g+rwx /dev/uinput
```

This assumes your user is member of the `wheel` group.

The commands must be rerun when new devices are connected. There might be a better way to set the permissions.

#### Set permissions automatically at boot

```sh
echo '@reboot root chmod -R g+rwx /dev/input' >> /etc/crontab
echo '@reboot root chmod g+rwx /dev/uinput' >> /etc/crontab
```

### Limitations and problems

#### Keyboard layout resets when xremap is started

To mitigate, replace `gb` with your keyboard layout, and run the command after xremap has started:

```sh
setxkbmap gb
```

[The problem is also described here](https://forums.freebsd.org/threads/keyboard-layout-keeps-getting-messed-up.95081/)

#### Key sequences, double tap and chords are not supported

The cause is missing `TimerFd` in [nix crate](https://github.com/nix-rust/nix). But that seems to
be added in [`v0.31.2`](https://github.com/nix-rust/nix/blob/master/CHANGELOG.md), so an update might fix that.

#### Config and device watching is not supported

The cause is missing `Inotify` in [nix crate](https://github.com/nix-rust/nix). There is
an alternative cross-platform crate [notify](https://crates.io/crates/notify).

#### LED events

LED events are not emitted from xremap, but it doesn't seem to cause any problems.
That is, `Capslock` and `NumLock` seems to work normally. The cause it that [evdev crate](https://github.com/emberian/evdev) doesn't support `with_led` for `VirtualDevice`.
