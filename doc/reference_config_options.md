## Configuration options

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

```yml
keypress_delay_ms: 10 # Default is `0`
# Rest of your config file
```

Some applications have trouble understanding synthesized key events, especially on
Wayland. `keypress_delay_ms` can be used to workaround the issue. Only events from
key combos in `keymap` are delayed. If that isn't enough try `throttle_ms`.

### throttle_ms

```yml
throttle_ms: 10 # Default is `0`
# Rest of your config file
```

Slow down synthetic events, so applications have the time to register events.
All events from anywhere in xremap are slowed down between:

- Press and release of the same key. But not the other way around.
- Press of ordinary key and press/release of modifier key.
- Press/release of modifier key and press of ordinary key.

The delay is only added if needed. That is there's no delay if the time has already elapsed.

The logic is choosen to allow applications enough time to register the exact modifiers pressed when
a key is pressed, and keep to key pressed enough time.

Since version 0.14.14.

### default_mode

```yml
default_mode: my_start_mode # Default is "default"
# Rest of your config file
```

This will be the value of `mode` when `xremap` starts. `mode` [is described here](../README.md#mode).

### notififactions

```yml
notififactions: true # Default is false
# Rest of your config file
```

With `true` will all desktop notifications from xremap be shown. This is currently a message when xremap
is loaded, when configuration files are reloaded and when reloading configuration files fails.

Since version 0.15.1.

### config_watch_debounce_ms

```yml
config_watch_debounce_ms: 20 # Default is '0'
# Rest of your config file
```

When using `--watch=config` to watch configuration files for changes, a delay can be needed to ignore
some of the file-change events. This can help an error where the old configuration is unloaded
when a configuration file with errors is saved.

By waiting for the inactivity to have remained e.g. `20ms` will only that final event
be used and therefore are all the 'wrong' events just ignored.

Since version 0.15.1.

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

### enable_wheel

```yml
enable_wheel: false # Default is "true"
# Rest of your config file
```

By default will the output device created by xremap support the mouse events `REL_X`, `REL_Y`, `REL_HWHEEL`, `REL_WHEEL`.
This does not affect the events/devices that are listened to, see the commandline argument `--mouse` for that.

With `enable_wheel: false` will the output device not support `REL_HWHEEL`, `REL_WHEEL`.
