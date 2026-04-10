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

Default is `0ms`.

Some applications have trouble understanding synthesized key events, especially on
Wayland. `keypress_delay_ms` can be used to workaround the issue. Only events from
key combos in `keymap` are delayed. If that isn't enough try `throttle_ms`.

### throttle_ms

Default is `0ms`.

Slow down synthetic events, so applications have the time to register events.
All events from anywhere in xremap are slowed down between:

- Press and release of the same key. But not the other way around.
- Press of ordinary key and press/release of modifier key.
- Press/release of modifier key and press of ordinary key.

The delay is only added if needed. That is there's no delay if the time has already elapsed.

The logic is choosen to allow applications enough time to register the exact modifiers pressed when
a key is pressed, and keep to key pressed enough time.

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
