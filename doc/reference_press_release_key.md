## Press/release key

The press/release key can be used to match when keys are pressed, released or repeated.
It's only available in `modmap`, and can as the only feature in `modmap` emit actions.

[In more detail](#in-more-detail)

### Example: Action on key press

This configuration will show a notification each time `KEY_A` is pressed. It will also
preserve the normal behavior of `KEY_A`.

```yml
modmap:
  - remap:
      A:
        press: { launch: ["notify-send", "KEY_A pressed"] }
```

### Example: Shift lock

When using `Capslock` it's possible to type uppercase letters, but numbers and symbols will keep their
ordinary behavior. ShiftLock is a technique to lock the keyboard so it's possible type uppercase letters
and symbols. It's a way to avoid holding shift.

```yml
modmap:
  - mode: LeftShiftLocked
    remap:
      LeftShift:
        skip_key_event: true
        press:
          - { release: LeftShift }
          - { set_mode: default }

  - remap:
      LeftShift:
        skip_key_event: true
        press:
          - { press: LeftShift }
          - { set_mode: LeftShiftLocked }
```

Note: It will still lock shift if it's interrupted. The normal way to use shift is to interrupt it.
So the normal behavior is not preserved with this example.

### Example: Emit key combo on key press

When `Capslock` is pressed will the key combo `ctrl-v` be emitted and it will be held until `Capslock`
is released. This is different from remapping in `keymap` with `Capslock: c-v`, where the key combo is released right
away.

```yml
modmap:
  - remap:
      Capslock:
        press: [{ press: LeftCtrl }, { press: V }]
        release: [{ release: V }, { release: LeftCtrl }]
        skip_key_event: true
```

### Example: Disable key repeat

This configuration makes the press/release explicit and then disables the normal action. The effect
is that the key doesn't do anything on repeat.

```yml
modmap:
  - remap:
      A:
        press: { press: A }
        release: { release: A }
        skip_key_event: true
```

Note: This doesn't necessarily disable the repeat behavior, because the kernel inserts key repeat
automatically for the latest pressed key.

## In more detail

`press`, `release` and `repeat` can all take the same values and default is no actions.

```yml
modmap:
  - remap:
      Capslock:
        skip_key_event: true # Default is false.
        press: KEY_A # Single key
        release: Ctrl-V # Key combo
        repeat: [KEY_B, KEY_C] # Several keys or key combos.
        # repeat: [] # Default is no actions.
        # repeat: null # Another way to say no actions.
        # repeat: [{ sleep: 10 }] # Single action
        # repeat: [{ press: D }, { release: D }] # Several actions.
```

### Actions

The same [actions as in keymap](#in-more-detail) are supported.

The trigger key is emitted after the actions, except if it's disabled with `skip_key_event`.

Key events emitted from `modmap` will normally go through `keymap` and can be remapped there. That's still true
for the trigger key. It's however not true for actions
that are emitted from press/release keys. They go around the `keymap` even if they contain key events like
`[KEY_A, { press: B }, { release: B }]`.

The `set_mode`, `escape_next_key` and `mark_set` take effect before the trigger key is remapped in `keymap`.
