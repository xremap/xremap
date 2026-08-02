## Actions

Actions can only be used in `keymap` and [press/release keys](reference_press_release_key.md).

### Without argument

| Name              | Description                                   | Added in |
| ----------------- | --------------------------------------------- | -------- |
| exit              | Close xremap gracefully                       | v0.15.10 |
| reload_config     | Reload configuration file, partially          | v0.15.10 |
| pop_window_info   | Show popup with window-info used for matching | v0.15.10 |
| print_window_info | Print window-info used for matching           | v0.15.10 |
| print_window_list | Print list of open windows                    | v0.15.10 |

The above actions are used like this: `TriggerKey: { action: name }`. Name is case-insensitive.

`reload_config` reloads the same way `--watch` does. It's a limited reload, because
reloading configuration in a running program is a hard thing to do. So it does a simple replacement
of `modmap`, `keymap` and `virtual_modifiers`. Everything else in `xremap` is left
as is. This includes for instance the `mode`.

Note: `print_window_list` is not supported for GNOME Wayland or KDE Wayland.

### With argument

| Name                      | Argument          | Description                            | Added in |
| ------------------------- | ----------------- | -------------------------------------- | -------- |
| press, repeat and release | Key               | Send the given key event               |          |
| launch                    | Vec&lt;String&gt; | Run a command                          |          |
| set_mode                  | String            | Set mode used to enable/disable remaps |          |
| set_mark                  | Boolean           | Enable/disable emacs mark-mode         |          |
| with_mark                 | KeyCombo          | Add shift to key combo if in mark-mode |          |
| escape_next_key           | Boolean           | Disable remapping for next key event   |          |
| sleep                     | Number            | Block all processing x milliseconds    | v0.10.4  |
| close_apps                | String            | Close programs with given app class    | v0.15.3  |

The above actions are used like this: `TriggerKey: { name: argument }`. Name is case-insensitive.

Note: `close_apps` is not supported for GNOME Wayland or Pantheon.

### Remap action (aka key sequence)

[Described seperately](./reference_key_sequence.md)

### Example: Run programs

Run a program when `KEY_A` is pressed (and repeated). The trigger key is disabled entirely.
The program is just started and ignored, it does not block key processing.

```yml
keymap:
  - remap:
      Capslock: { launch: ["notify-send", "Hello World!"] }
```

Note: Holding down `capslock` will generate repeat events, which will run the program every say `50ms`.

### Example: Run bash script

By using yaml notation for multi-line text it's possible to write scripts:

```yml
keymap:
  - remap:
      Capslock:
        launch:
          - "bash"
          - "-c"
          - |
            NAME=`whoami`
            WORKDIR=`pwd`
            notify-send "Username: $NAME" "Workdir: $WORKDIR"
```

### Example: Key events

It's possible to emit the individual press, repeat and release events:

```yml
throttle_ms: 10 # Slows down events.
keymap:
  - remap:
      Capslock: [{ press: B }, { repeat: B }, { release: B }]
```

Note: `throttle_ms` is necessary for some applications because actions are
sent without any delay in-between by default.

Note: That repeat events are usually ignored by the kernel. It emits the repeat events itself.

### Example: Sleep

Block key processing in the given amount of `ms`.

```yml
keymap:
  - remap:
      Capslock: { sleep: 10 }
```

### Example: Close applications

Close all applications that have the exact `app_class`.

```yml
keymap:
  - remap:
      Capslock: { close_apps: "firefox" }
```

Since: `v0.15.3`. Not supported in GNOME Wayland or Pantheon.
