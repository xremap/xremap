## Feature matrix

- 🟢 Works
- 🟡 Partial
- 🔵 Is possible
- ⚪ Might be possible
- 🔴 Not possible

### Desktop integration

| Feature                                   | X11 | GNOME Wayland | Hyprland | KDE Wayland | wlroots | Niri | COSMIC | socket | Pantheon |
| ----------------------------------------- | --- | ------------- | -------- | ----------- | ------- | ---- | ------ | ------ | -------- |
| notifications                             | 🟢  | 🟢            | 🟢       | 🟢          | 🟢      | 🟢   | 🟢     | 🟢     | 🟢       |
| print_window_list<br>and `--list-windows` | 🟢  | 🟡[1]         | 🟢       | 🟡[1]       | 🟢      | 🟢   | 🟢     | 🟢     | 🟢       |
| close_apps                                | 🟢  | 🔵            | 🟢       | 🟢          | 🟢      | 🟢   | 🟢     | 🟢     | 🔴       |

[1] It's possible to use `print_window_info` instead.

## Operating system

| Feature                         | Linux | FreeBSD | OpenBSD/NetBSD |
| ------------------------------- | ----- | ------- | -------------- |
| Standard key remapping          | 🟢    | 🟢      | 🔴[1]          |
| Aplication-specific remapping   | 🟢    | 🟢      |                |
| Watching for new keyboards/mice | 🟢    | 🔵      |                |
| Watching config file            | 🟢    | 🔵      |                |

[1] OpenBSD or NetBSD does not have `evdev` or an equivalent way to remap keys.
