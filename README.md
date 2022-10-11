# 𝑋𝑟𝑒𝑚𝑎𝑝 :keyboard: [![cargo](https://github.com/k0kubun/xremap/actions/workflows/build.yml/badge.svg)](https://github.com/k0kubun/xremap/actions/workflows/build.yml)

`xremap` is a key remapper for Linux. Unlike `xmodmap`, it supports app-specific remapping and Wayland.

## Concept

* **Fast** - Xremap is written in Rust, which is faster than JIT-less interpreters like Python.

* **Cross-platform** - Xremap uses `evdev` and `uinput`, which works whether you use X11 or Wayland.

* **Language-agnostic** - The config is JSON-compatible. Generate it from any language,
  e.g. [Ruby](https://github.com/xremap/xremap-ruby), [Python](https://github.com/xremap/xremap-python).

## Features

* Remap any keys, e.g. Ctrl or CapsLock.
* Remap any key combination to another, even to a key sequence.
* Remap a key sequence as well. You could do something like Emacs's `C-x C-c`.
* Remap a key to two different keys depending on whether it's pressed alone or held.
* Application-specific remapping. Even if it's not supported by your application, xremap can.
* Automatically remap newly connected devices by starting xremap with `--watch`.
* Support [Emacs-like key remapping](example/emacs.yml), including the mark mode.
* Trigger commands on key press/release events.
* Use a non-modifier key as a virtual modifier key.

## Installation

Download a binary from [Releases](https://github.com/k0kubun/xremap/releases).

If it doesn't work, please [install Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
and run one of the following commands:

```bash
cargo install xremap --features x11   # X11
cargo install xremap --features gnome # GNOME Wayland
cargo install xremap --features sway  # Sway
cargo install xremap                  # Others
```

You may also need to install `libx11-dev` to run the `xremap` binary for X11.

### Arch Linux

If you are on Arch Linux and X11, you can install [xremap-x11-bin](https://aur.archlinux.org/packages/xremap-x11-bin/) from AUR.

### NixOS

If you are using NixOS, xremap can be installed and configured through a [flake](https://github.com/xremap/nix-flake/).

## Usage

Write [a config file](#Configuration) directly, or generate it with
[xremap-ruby](https://github.com/xremap/xremap-ruby) or [xremap-python](https://github.com/xremap/xremap-python).
Then run:

```
sudo xremap config.yml
```

<details>
<summary>If you want to run xremap without sudo, click here.</summary>

### Running xremap without sudo

To do so, your normal user should be able to use `evdev` and `uinput` without sudo.
In Ubuntu, this can be configured by running the following commands and rebooting your machine.

```bash
sudo gpasswd -a YOUR_USER input
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/input.rules
```

#### Arch Linux

The following can be used on Arch.

```bash
lsmod | grep uinput
```
If this module is not loaded, add to `/etc/modules-load.d/uinput.conf`:
```bash
uinput
```
Then add udev rule.

```bash
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/99-input.rules
```

#### Other platforms

In other platforms, you might need to create an `input` group first
and run `echo 'KERNEL=="event*", NAME="input/%k", MODE="660", GROUP="input"' | sudo tee /etc/udev/rules.d/input.rules` as well.

If you do this, in some environments, `--watch` may fail to recognize new devices due to temporary permission issues.
Using `sudo` might be more useful in such cases.

---

</details>

See the following instructions for your environment to make `application`-specific remapping work.

### X11

If you use `sudo` to run `xremap`, you may need to run `xhost +SI:localuser:root` if you see `No protocol specified`.

### GNOME Wayland

Install xremap's GNOME Shell extension from [this link](https://extensions.gnome.org/extension/5060/xremap/),
switching OFF to ON.

<details>
<summary>If you use <code>sudo</code> to run <code>xremap</code>, also click here.</summary>

Update `/usr/share/dbus-1/session.conf` as follows, and reboot your machine.

```diff
   <policy context="default">
+    <allow user="root"/>
     <!-- Allow everything to be sent -->
     <allow send_destination="*" eavesdrop="true"/>
     <!-- Allow everything to be received -->
```

</details>

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

See also: [example/config.yml](example/config.yml) and [example/emacs.yml](example/emacs.yml)

### modmap

`modmap` is for key-to-key remapping like xmodmap.
Note that remapping a key to a modifier key, e.g. CapsLock to Control\_L,
is supported only in `modmap` since `keymap` handles modifier keys differently.

```yml
modmap:
  - name: Name # Optional
    remap: # Required
      # Replace a key with another
      KEY_XXX: KEY_YYY # Required
      # Dispatch different keys depending on whether you hold it or press it alone
      KEY_XXX:
        held: KEY_YYY # Required
        alone: KEY_ZZZ # Required
        alone_timeout_millis: 1000 # Optional
      # Hook `keymap` action on key press/release events.
      KEY_XXX:
        press: { launch: ["xdotool", "mousemove", "0", "7200"] } # Required
        release: { launch: ["xdotool", "mousemove", "0", "0"] } # Required
    application: # Optional
      not: [Application, ...]
      # or
      only: [Application, ...]
```

For `KEY_XXX` and `KEY_YYY`, use [these names](https://github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/scancodes.rs#L26-L572).
You can skip `KEY_` and the name is case-insensitive. So `KEY_CAPSLOCK`, `CAPSLOCK`, and `CapsLock` are the same thing.
Some [custom aliases](src/config/key.rs) like `SHIFT_R`, `CONTROL_L`, etc. are provided.

If you specify a map containing `held` and `alone`, you can use the key for two purposes.
The key is considered `alone` if it's pressed and released within `alone_timeout_millis` (default: 1000)
before any other key is pressed. Otherwise it's considered `held`.

### keymap

`keymap` is for remapping a sequence of key combinations to another sequence of key combinations or other actions.

```yml
keymap:
  - name: Name # Optional
    remap: # Required
      # Key press -> Key press
      MOD1-KEY_XXX: MOD2-KEY_YYY
      # Sequence (MOD1-KEY_XXX, MOD2-KEY_YYY) -> Key press (MOD3-KEY_ZZZ)
      MOD1-KEY_XXX:
        remap:
          MOD2-KEY_YYY: MOD3-KEY_ZZZ
        timeout_millis: 200 # Optional. No timeout by default.
      # Key press (MOD1-KEY_XXX) -> Sequence (MOD2-KEY_YYY, MOD3-KEY_ZZZ)
      MOD1-KEY_XXX: [MOD2-KEY_YYY, MOD3-KEY_ZZZ]
      # Execute a command
      MOD1-KEY_XXX:
        launch: ["bash", "-c", "echo hello > /tmp/test"]
      # Let `with_mark` also press a Shift key (useful for Emacs emulation)
      MOD1-KEY_XXX: { set_mark: true } # use { set_mark: false } to disable it
      # Also press Shift only when { set_mark: true } is used before
      MOD1-KEY_XXX: { with_mark: MOD2-KEY_YYY }
      # The next key press will ignore keymap
      MOD1-KEY_XXX: { escape_next_key: true }
      # Set mode to configure Vim-like modal remapping
      MOD1-KEY_XXX: { set_mode: default }
    application: # Optional
      not: [Application, ...]
      # or
      only: [Application, ...]
    mode: default # Optional
default_mode: default # Optional
```

For `KEY_XXX`, use [these names](https://github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/scancodes.rs#L26-L572).
You can skip `KEY_` and the name is case-insensitive. So `KEY_CAPSLOCK`, `CAPSLOCK`, and `CapsLock` are the same thing.

For the `MOD1-` part, the following prefixes can be used (also case-insensitive):

* Shift: `SHIFT-`
* Control: `C-`, `CTRL-`, `CONTROL-`
* Alt: `M-`, `ALT-`
* Windows: `SUPER-`, `WIN-`, `WINDOWS-`

You can use multiple prefixes like `C-M-Shift-a`.
You may also suffix them with `_L` or `_R` (case-insensitive) so that
remapping is triggered only on a left or right modifier, e.g. `Ctrl_L-a`.

If you use `virtual_modifiers` explained below, you can use it in the `MOD1-` part too.

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

The application name can be specified as a normal string to exactly match the name,
or a regex surrounded by `/`s like `/application/`.

To check the application names, you can use the following commands:

#### X11

```
$ wmctrl -x -l
0x02800003  0 slack.Slack           ubuntu-jammy Slack | general | ruby-jp
0x05400003  0 code.Code             ubuntu-jammy application.rs - xremap - Visual Studio Code
```

You may use the entire string of the third column (`slack.Slack`, `code.Code`),
or just the last segment after `.` (`Slack`, `Code`).

#### GNOME Wayland

```
busctl --user call org.gnome.Shell /com/k0kubun/Xremap com.k0kubun.Xremap WMClass
```

#### Sway

```
swaymsg -t get_tree
```

Locate `app_id` in the output.

### virtual\_modifiers

You can declare keys that should act like a modifier.

```yml
virtual_modifiers:
  - CapsLock
keymap:
  - remap:
      CapsLock-i: Up
      CapsLock-j: Left
      CapsLock-k: Down
      CapsLock-l: Right
```

### keypress_delay_ms

Some applications have trouble understanding synthesized key events, especially on
Wayland. `keypress_delay_ms` can be used to workaround the issue.
See [#179](https://github.com/k0kubun/xremap/issues/179) for the detail.

## License

`xremap` is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
