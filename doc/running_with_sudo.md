# Running xremap with sudo

### Pro

- It's the easiest way to getting started with `xremap`. You don't need to add any permissions to your normal user.

### Con

- If you want to use application-specific remappings there are extra steps to take.
- If you launch programs from xremap they will run as `root`, not as your own normal user. It's considered a security risk to do so.
- If the config file can be edited by other users than `root`, they can effectively run any command as `root` by changing your config file.

## Without application-specific remapping

It's plain and simple:

```
sudo xremap config.yml
```

## With application-specific remapping

### X11

Run this command:

```
sudo xremap config.yml
```

You may need to run `xhost +SI:localuser:root` if you see `No protocol specified`.

### GNOME Wayland

Install xremap's GNOME Shell extension from [this link](https://extensions.gnome.org/extension/5060/xremap/),
switching OFF to ON.

Update `/usr/share/dbus-1/session.conf` as follows, and reboot your machine.

```diff
   <policy context="default">
+    <allow user="root"/>
     <!-- Allow everything to be sent -->
     <allow send_destination="*" eavesdrop="true"/>
     <!-- Allow everything to be received -->
```

Then run:

```
sudo -E xremap config.yml
```

If you don't like transferring all the environment variables to root you could try the alternative below.

### KDE-Plasma Wayland

Xremap cannot be run as root. [There are other ways to run xremap](./README.md)

### Niri, Sway and COSMIC Wayland

Run this command:

```
sudo -E xremap config.yml
```

This transfers all the environment variables to the root user, so it knows how to connect to your desktop session.

Whether there's a drawback to doing this is unknown, and you can opt for just transferring the minimal environment variables. Shown below.

#### Alternative for Niri

```bash
sudo NIRI_SOCKET="$NIRI_SOCKET" xremap config.yml
```

#### Alternative for Sway

```sh
sudo env WAYLAND_DISPLAY="$WAYLAND_DISPLAY" XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" SWAYSOCK="$SWAYSOCK" xremap config.yml
```

#### Alternative for COSMIC Wayland

```sh
sudo env WAYLAND_DISPLAY="$WAYLAND_DISPLAY" XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" xremap config.yml
```

#### Alternative for GNOME Wayland

```sh
sudo env DBUS_SESSION_BUS_ADDRESS="$DBUS_SESSION_BUS_ADDRESS" XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" xremap config.yml
```
