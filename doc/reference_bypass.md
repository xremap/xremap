## Bypass

After events have been remapped will the output bypass all further processing.
In other words, they will only be remapped once.

By default will all events go through all three remaps in the following
order: `experimental_map` -> `modmap` -> `keymap`. And can be remapped in all three places.

Working since v0.15.15

### Global default

```yml
bypass: true # Default is false
# Rest of your config file
```

This value applies to all remaps in both `modmap` and `experimental_map`.
`keymap` isn't relevant because there's nothing to bypass after `keymap`.

### Group level

If bypass is set on the group level, then that value is used for all
remaps in that group, ignoring the global setting for that group.

#### Example: Emit virtual modifier key

Without bypass can virtual modifiers not be emitted from `modmap`, because `keymap` filters them out.

By using bypass:

```yml
virtual_modifiers:
  - capslock
modmap:
  - bypass: true
    remap:
      f9: capslock
keymap:
  - remap:
      capslock-J: DOWN
```

When pressing `f9` will `capslock` be emitted, because bypass makes it go
around keymap, and therefore not filtered like normal.

Note that `f9-J` will also emit `down`, because even though `capslock` bypasses
keymap will it still be used as modifiers for later events.

### Specification

- The delays configured with [throttle_ms](reference_config_options.md#throttle_ms) will still be applied.
- Bypass events will not interrupt other actions, e.g:
  - Bypass output from `modmap` and `experimental_map` will not interrupt a
    [key sequence](reference_key_sequence.md) in `keymap`.
  - Bypass output from `experimental_map` will not interrupt multi purpose keys in `modmap`,
    where they're normally interrupted and cause a hold-action.
- Bypassed modifier keys will still be used in determining whether key combos match.
- Keys declared as virtual modifiers will also be emitted with bypass=true. Normally they're filtered out in keymap.
