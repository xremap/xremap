# 𝑋𝑟𝑒𝑚𝑎𝑝 :keyboard: [![cargo](https://github.com/k0kubun/xremap/actions/workflows/build.yml/badge.svg)](https://github.com/k0kubun/xremap/actions/workflows/build.yml)

`xremap` is a key remapper for Linux. Unlike `xmodmap`, it supports app-specific remapping and Wayland.

## Features

* You can remap any keys, e.g. Ctrl or CapsLock.
* You can remap any key combination to another, even to a key sequence
* You can also remap a key sequence as well. You could do something like Emacs's `C-x C-c`.
* Application-specific remapping. Even if it's not supported by your application, xremap can.

## Concept

* **Fast** - Xremap is written in Rust, which is faster than JIT-less interpreters like Python.

* **Cross-platform** - Xremap uses `evdev` and `uinput`, which works whether you use X11 or Wayland.

* **Language-agnostic** - The config is JSON-compatible. Generate it from any language,
  e.g. [Ruby](https://github.com/xremap/xremap-ruby), [Python](https://github.com/xremap/xremap-python).

## Prerequisite

`xremap` assumes that you can use `evdev` and `uinput` without sudo.
You can configure it as follows:

```bash
sudo gpasswd -a YOUR_USER input
echo 'KERNEL=="uinput", GROUP="input"' | sudo tee /etc/udev/rules.d/input.rules
```

Then reboot your machine to make sure the udev rule is applied.

## Installation

After the reboot, download a binary from [Releases](https://github.com/k0kubun/xremap/releases).

If it doesn't work, please [install Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
and run one of the following commands in this repository and use `target/release/xremap`:

```bash
# X11
cargo build xremap --release --features x11

# GNOME Wayland
cargo build xremap --release --features gnome

# Sway
cargo build xremap --release --features sway

# Others
cargo build xremap --release
```

You may also need to install `libx11-dev` to run the `xremap` binary for X11.

## Usage

Write [a config file](#Configuration) directly, or generate it with
[xremap-ruby](https://github.com/xremap/xremap-ruby) or [xremap-python](https://github.com/xremap/xremap-python).
Then run:

```
xremap config.yml
```

### Dynamic binding

Xremap supports application-specific key remapping.

While Xremap uses `evdev` and `uinput`, which is a lower layer than X11 and Wayland,
Xremap also uses X11 or Wayland compositor-specific protocols to support `application` config.
If you need this feature, make sure you specify the correct binary or `--features` option,
and pay attention to the error messages from `xremap`.

## Configuration
Your `config.yml` should look like this:

```yml
modmap:
  - name: Except Chrome
    application:
      not: Google-chrome
    remap:
      CapsLock: Esc
keymap:
  - name: Emacs binding
    application:
      only: Slack
    remap:
      C-b: left
      C-f: right
      C-p: up
      C-n: down
```

See also: [example/config.yml](./example/config.yml)

### modmap

`modmap` is for key-to-key remapping like xmodmap.
If you want to remap modifier keys, you need to use `modmap`.
Note that `modmap` remapping happens before `keymap` remapping.

```yml
modmap:
  - name: Name # Required
    remap: # Required
      KEY_XXX: KEY_YYY
    application: # Optional
      not: [Application, ...]
      # or
      only: [Application, ...]
```

For `KEY_XXX` and `KEY_YYY`, use [these names](https://github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/scancodes.rs#L26-L572).
You can skip `KEY_` and the name is case-insensitive. So `KEY_CAPSLOCK`, `CAPSLOCK`, and `CapsLock` are the same thing.
Some [custom aliases](src/config/key.rs) like `SHIFT_R`, `CONTROL_L`, etc. are provided.

### keymap

`modmap` is for remapping a sequence of key combinations to another sequence of key combinations or other actions.

```yml
modmap:
  - name: Name # Required
    remap: # Required
      # key press -> key press
      MOD1-KEY_XXX: MOD2-KEY_YYY
      # sequence (MOD1-KEY_XXX, MOD2-KEY_YYY) -> key press (MOD3-KEY_ZZZ)
      MOD1-KEY_XXX:
        remap:
          MOD2-KEY_YYY: MOD3-KEY_ZZZ
      # key press (MOD1-KEY_XXX) -> sequence (MOD2-KEY_YYY, MOD3-KEY_ZZZ)
      MOD1-KEY_XXX: [MOD2-KEY_YYY, MOD3-KEY_ZZZ]
    application: # Optional
      not: [Application, ...]
      # or
      only: [Application, ...]
```

For `KEY_XXX`, use [these names](https://github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/scancodes.rs#L26-L572).
You can skip `KEY_` and the name is case-insensitive. So `KEY_CAPSLOCK`, `CAPSLOCK`, and `CapsLock` are the same thing.

For the `MOD1-` part, the following prefixes can be used (also case-insensitive):

* Shift: `SHIFT-`
* Control: `C-`, `CTRL-`, `CONTROL-`
* Alt: `M-`, `ALT-`
* Windows: `SUPER-`, `WIN-`, `WINDOWS-`

You may use multiple prefixes like `C-M-Shift-a`.

### application

`application` can be used for both `modmap` and `keymap`, which allows you to specify application-specific remapping.

```yml
application:
  not: Application
  # or
  not: [Application, ...]
  # or
  only: Application
  # or
  only: [Application, ...]
```

To check the application names, you can use the following commands:

#### X11

```
$ wmctrl -x -l
0x0280000a  0 gnome-terminal-server.Gnome-terminal  ubuntu-focal Terminal
0x02600001  0 nocturn.Nocturn       ubuntu-focal Nocturn
```

Use the name after `.` in the third column (`WM_CLASS`), i.e. `Gnome-terminal` or `Nocturn` in the above output.

#### GNOME Wayland

```
busctl --user call org.gnome.Shell /org/gnome/Shell org.gnome.Shell Eval s 'global.get_window_actors().map(a => a.get_meta_window().get_wm_class());'
```

#### Sway

```
swaymsg -t get_tree
```

Locate `app_id` in the output.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
