# Running xremap with sudo

When you run with sudo you don't need to setup extra permissions. But if you want to use
application-specific remappings, there are extra steps to take depending on which desktop environment you use.

See the following instructions for your environment to make `application`-specific remapping work.

Run this command, when you stop the command everything will be back to normal.

```
sudo xremap config.yml
```

## X11

If you use `sudo` to run `xremap`, you may need to run `xhost +SI:localuser:root` if you see `No protocol specified`.

## GNOME Wayland

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

## KDE-Plasma Wayland

Xremap cannot be run as root. Follow the instructions above to run xremap without sudo.

## Niri

If you use `sudo` to run `xremap`, you need to ensure that the `NIRI_SOCKET` env var is available to xremap:

```bash
sudo NIRI_SOCKET="$NIRI_SOCKET" xremap config.yml
```
