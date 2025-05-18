# <img src='.github/xremap.png' style='height: 32px; margin-top: 8px; margin-bottom: -4px;' alt='Xremap'> :keyboard:

![crates.io](https://img.shields.io/crates/v/xremap) [![GitHub Actions](https://github.com/k0kubun/xremap/actions/workflows/build.yml/badge.svg)](https://github.com/k0kubun/xremap/actions/workflows/build.yml)

`xremap` is a key remapper for Linux. Unlike `xmodmap`, it supports app-specific remapping and Wayland.

## Concept

- **Fast** - Xremap is written in Rust, which is faster than JIT-less interpreters like Python.

- **Cross-platform** - Xremap uses `evdev` and `uinput`, which works whether you use X11 or Wayland.

- **Language-agnostic** - The config is JSON-compatible. Generate it from any language,
  e.g. [Ruby](https://github.com/xremap/xremap-ruby), [Python](https://github.com/xremap/xremap-python).

## Features

- Remap any keys, e.g. Ctrl or CapsLock.
- Remap any key combination to another, even to a key sequence.
- Remap a key sequence as well. You could do something like Emacs's `C-x C-c`.
- Remap a key to two different keys depending on whether it's pressed alone or held.
- Application-specific remapping. Even if it's not supported by your application, xremap can.
- Device-specific remapping.
- Automatically remap newly connected devices by starting xremap with `--watch`.
- Support [Emacs-like key remapping](example/emacs.yml), including the mark mode.
- Trigger commands on key press/release events.
- Use a non-modifier key as a virtual modifier key.

## Installation

Download a binary from [Releases](https://github.com/k0kubun/xremap/releases).

If it doesn't work, please [install Rust](https://doc.rust-lang.org/cargo/getting-started/installation.html)
and run one of the following commands:

```bash
cargo install xremap --features x11     # X11
cargo install xremap --features gnome   # GNOME Wayland
cargo install xremap --features kde     # KDE-Plasma Wayland
cargo install xremap --features wlroots # Sway, Wayfire, etc.
cargo install xremap --features hypr    # Hyprland
cargo install xremap                    # Others
```

You may also need to install `libx11-dev` to run the `xremap` binary for X11.

### Arch Linux

If you are on Arch Linux and X11, you can install [xremap-x11-bin](https://aur.archlinux.org/packages/xremap-x11-bin/) from AUR.

### NixOS

If you are using NixOS, xremap can be installed and configured through a [flake](https://github.com/xremap/nix-flake/).

### Fedora Linux

If you are using Fedora, xremap can be installed via this [Fedora Copr](https://copr.fedorainfracloud.org/coprs/blakegardner/xremap/) repository.

## Usage

Write [a config file](#Configuration) directly, or generate it with
[xremap-ruby](https://github.com/xremap/xremap-ruby) or [xremap-python](https://github.com/xremap/xremap-python).

Then start the `xremap` daemon by running:

```
sudo xremap config.yml
```

(You will need to leave it running for your mappings to take effect.)

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
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/99-input.rules
```

Then reboot the machine.

#### Debian

Make sure `uinput` is loaded same as in Arch:

```
lsmod | grep uinput
```

If it shows up empty:

```bash
echo uinput | sudo tee /etc/modules-load.d/uinput.conf
```

Add your user to the `input` group and add the same udev rule as in Ubuntu:

```bash
sudo gpasswd -a YOUR_USER input
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/input.rules
```

Reboot the machine afterwards or try:

```bash
sudo modprobe uinput
sudo udevadm control --reload-rules && sudo udevadm trigger
```

#### NixOS

The following can be used on NixOS.

Ensure `uninput` is enabled in your `configuration.nix`:

```nix
hardward.uinput.enable = true;
boot.kernelModules = [ "uinput" ];
```

Then add the rule to the `udev` extra rules in your `configuration.nix`:

```nix
services.udev.extraRules = ''
  KERNEL=="uinput", GROUP="input", TAG+="uaccess"
  '';
```

The new rule will be added to `/etc/udev/rules.d/99-local.rules`. See [NixOS documentation](https://search.nixos.org/options?channel=24.11&show=services.udev.extraRules&from=0&size=50&sort=relevance&type=packages&query=services.udev) for additional information.

Rebuild with `nixos-rebuild switch`. Note you may also need to reboot your machine.

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

### KDE-Plasma Wayland

Xremap cannot be run as root. Follow the instructions above to run xremap without sudo.

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
Note that remapping a key to a modifier key, e.g. CapsLock to Control_L,
is supported only in `modmap` since `keymap` handles modifier keys differently.

```yml
modmap:
  - name: Name # Optional
    exact_match: false # Optional, defaults to false
    remap: # Required
      # Replace a key with another
      KEY_XXX1: KEY_YYY # Required
      # Dispatch different keys depending on whether you hold it or press it alone
      KEY_XXX2:
        held: KEY_YYY # Required, also accepts arrays
        alone: KEY_ZZZ # Required, also accepts arrays
        alone_timeout_millis: 1000 # Optional
      # Hook `keymap` action on key press/release events.
      KEY_XXX3:
        skip_key_event: true # Optional, skip original key event, defaults to false
        press: [{ press: KEY_YYY }, { launch: ["xdotool", "mousemove", "0", "7200"] }] # Optional
        repeat: { repeat: KEY_YYY } # Optional
        release: [{ release: KEY_YYY }, { set_mode: my_mode }] # Optional
    application: # Optional
      not: [Application, ...]
      # or
      only: [Application, ...]
    window: # Optional (only hyprland/wlroots/kde clients supported)
      not: [/regex of window title/, ...]
      # or
      only: [/regex of window title/, ...]
    device: # Optional
      not: [Device, ...]
      # or
      only: [Device, ...]
    mode: default # Optional
    # or
    mode: [ default, my_mode ]
default_mode: default # Optional
```

For `KEY_XXX` and `KEY_YYY`, use [these names](https://github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/scancodes.rs#L26-L572).
You can skip `KEY_` and the name is case-insensitive. So `KEY_CAPSLOCK`, `CAPSLOCK`, and `CapsLock` are the same thing.
Some [custom aliases](src/config/key.rs) like `SHIFT_R`, `CONTROL_L`, etc. are provided.

In case you don't know the name of a key, you can find out by enabling the xremap debug output:

```bash
RUST_LOG=debug xremap config.yml
# or
sudo RUST_LOG=debug xremap config.yml
```

Then press the key you want to know the name of.

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
      MOD1-KEY_XXX1: MOD2-KEY_YYY
      # Sequence (MOD1-KEY_XXX2, MOD2-KEY_YYY) -> Key press (MOD3-KEY_ZZZ)
      MOD1-KEY_XXX2:
        remap:
          MOD2-KEY_YYY: MOD3-KEY_ZZZ
        timeout_millis: 200 # Optional. No timeout by default.
      # Key press (MOD1-KEY_XXX3) -> Sequence (MOD2-KEY_YYY, MOD3-KEY_ZZZ)
      MOD1-KEY_XXX3: [MOD2-KEY_YYY, MOD3-KEY_ZZZ]
      # Execute a command
      MOD1-KEY_XXX4:
        launch: ["bash", "-c", "echo hello > /tmp/test"]
      # Let `with_mark` also press a Shift key (useful for Emacs emulation)
      MOD1-KEY_XXX5: { set_mark: true } # use { set_mark: false } to disable it
      # Also press Shift only when { set_mark: true } is used before
      MOD1-KEY_XXX6: { with_mark: MOD2-KEY_YYY }
      # After pressing MOD1-KEY_XXX7, the next key press will ignore keymap
      MOD1-KEY_XXX7: { escape_next_key: true }
      # Set mode to configure Vim-like modal remapping
      MOD1-KEY_XXX8: { set_mode: default }
      # Illustrate a nested mapping that times out;
      # also useful for timing out double-key sequences if the second key is never pressed.
      space:  # Use timeout to fix a bouncy spacebar
        remap:
          space: null          # make space output nothing; null is equivalent to []
          timeout_key: space   # output space after timeout or a non-mapped key (only space is mapped above)
          timeout_millis: 150  # timeout duration in ms
    application: # Optional
      not: [Application, ...]
      # or
      only: [Application, ...]
    window: # Optional (only hyprland/wlroots/kde clients supported)
      not: [/regex of window title/, ...]
      # or
      only: [/regex of window title/, ...]
    device: # Optional
      not: [Device, ...]
      # or
      only: [Device, ...]
    mode: default # Optional
    # or
    mode: [ default, my_mode ]
default_mode: default # Optional
```

For `KEY_XXX`, use [these names](https://github.com/emberian/evdev/blob/1d020f11b283b0648427a2844b6b980f1a268221/src/scancodes.rs#L26-L572).
You can skip `KEY_` and the name is case-insensitive. So `KEY_CAPSLOCK`, `CAPSLOCK`, and `CapsLock` are the same thing.

For the `MOD1-` part, the following prefixes can be used (also case-insensitive):

- Shift: `SHIFT-`
- Control: `C-`, `CTRL-`, `CONTROL-`
- Alt: `M-`, `ALT-`
- Windows: `SUPER-`, `WIN-`, `WINDOWS-`

You can use multiple prefixes like `C-M-Shift-a`.
You may also suffix them with `_L` or `_R` (case-insensitive) so that
remapping is triggered only on a left or right modifier, e.g. `Ctrl_L-a`.

If you use `virtual_modifiers` explained below, you can use it in the `MOD1-` part too.

`exact_match` defines whether to use exact match when matching key presses. For
example, given a mapping of `C-n: down` and `exact_match: false` (default), and
you pressed <kbd>C-Shift-n</kbd>, it will automatically be remapped to
<kbd>Shift-down</kbd>, without you having to define a mapping for
<kbd>C-Shift-n</kbd>, which you would have to do if you use `exact_match: true`.

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

Use the following command or check windows' WMClass by pressing Alt+F2 and running `lg` command in [LookingGlass](https://wiki.gnome.org/Projects/GnomeShell/LookingGlass):

```
busctl --user call org.gnome.Shell /com/k0kubun/Xremap com.k0kubun.Xremap WMClasses
```

#### KDE-Plasma Wayland

Xremap prints the active window to the console.
However, it will only start printing, once a mapping has been triggered that uses an application filter.
So you have to create a mapping with a filter using a dummy application name and trigger it.
Then each time you switch to a new window xremap will print its caption, class, and name in the following style:
`active window: caption: '<caption>', class: '<class>', name: '<name>'`
The `class` property should be used for application matching, while the `caption` property should be used for window matching.

If you use a systemd-daemon to manage xremap, the prints will be visible in the system-logs (Can be opened with `journalctl -f`)

#### Sway

```
swaymsg -t get_tree
```

Locate `app_id` in the output.

#### application-specific key overrides

Sometimes you want to define a generic key map that is available in all applications, but give specific keys in that map their own definition in specific applications. You can do this by putting the generic map at the bottom of the config, after any specific overrides, as follows.

```yml
# Emacs-style word-forward and word-back
keymap:
  - name: override to make libreoffice-writer go to end of word but before final space like emacs
    application:
      only: libreoffice-writter
    remap:
      Alt-f: [right, C-right, left]
  - name: generic for all apps
    remap:
      Alt-f: C-right
      Alt-b: C-left
```

Note how Alt-f and Alt-b work in all apps, but the definition of Alt-f is slightly different in LibreOffice Writer. When that app is active, the first definition overrides the second definition; but for any other app, only the second definition is found. This is because xremap uses the first matching definition that it finds.

### device

Much like [`application`](#application), you may specify `{keymap,modmap}.device.{not,only}` in your configuration for device-specific remapping. Consistent with the global `--device` flag, device-matching strings may be any of:

- the full path of the device
- the filename of the device
- the device name
- a substring of the device name

To determine the names and paths of your devices, examine `xremap`'s log output at startup.

```yml
device:
  not: '/dev/input/event0'
  # or
  not: ['event0', ...]
  # or
  only: 'Some Cool Device Name'
  # or
  only: ['Cool Device', ...]
  # etc...
```

Unlike for `application`, regexs are not supported for `device`.

### mode

You can assign mode(s) to keymap and/or remap which effectively turns them on or off
when you set the mode.

```yml
modmap:
  - name: Up
    remap:
      W: UP
    mode: [Up, Up_And_Down] # Mode is optional

  - name: Down
    remap:
      S: DOWN
    mode: [Down, Up_And_Down]

  - name: Right_And_Left
    remap:
      D: RIGHT
      A: LEFT
    mode: Right_And_Left # Mode can be a string or vector of strings

  - name: Turn Off
    remap:
      L:
        press: { set_mode: Off } # Modmap can set mode via press and release
        release:
    # If mode is absent the keymap or modmap is always on

keymap:
  - name: SetMode
    remap:
      CTRL-U: { set_mode: Up }
      CTRL-I: { set_mode: Down }
      CTRL-O: { set_mode: Up_And_Down }
      CTRL-P: { set_mode: Right_And_Left }
    mode: [Up, Down, Right_And_Left, Up_And_Down, Off] # You can assign modes to keymap too!

default_mode: Up_And_Down # Optional, if absent default mode is "default"
```

### virtual_modifiers

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

### Shared data field

You can declare data that does not directly go into the config under the `shared` field.  
This can be usefull when using Anchors and Aliases.  
For more information about the use of Yaml anchors see the [Yaml specification](https://yaml.org/spec/1.2.2/#3222-anchors-and-aliases).

#### example:

```yaml
shared:
  terminals: &terminals # The & Symbol marks this entry as a Anchor
    - Gnome-terminal
    - Kitty

  some_remaps: &some_remaps
    Ctrl-f: C-right
    Alt-b: C-up

keymap:
  - application:
      only: *terminals # we can reuse the list here
    remap: *some_remaps # and we can reuse a map here.
```

## Running xremap as a daemon

Put your config file at `~/.config/xremap/config.yml` and
copy `example/xremap.service` to `~/.config/systemd/user/xremap.service`.

```bash
cp example/xremap.service ~/.config/systemd/user/xremap.service
```

> [!WARNING]
> make sure `xremap` installaion path matches `xremap.service` path

then run

```bash
systemctl --user start xremap.service
```

To start the service on boot, `systemctl --user enable xremap.service` may sometimes work.
However, it may fail to recognize the window manager if you start xremap too early.
Consider copying `example/xremap.desktop` to `~/.config/autostart/xremap.desktop` if the platform supports it.

## Maintainers

- @k0kubun
- @N4tus (KDE client)
- @jixiuf (wlroots client)

## License

`xremap` is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
