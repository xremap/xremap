# Use normal key a modifier

To use a normal key like `Capslock` as a modifier it needs
to be declared in `virtual_modifiers`, which also means its normal behavior is suppressed.
It must be filtered out, to make sure pressing `Capslock-L` only emits `left`.

```yml
virtual_modifiers:
  - Capslock

keymap:
  - remap:
      Capslock-H: left
      Capslock-J: down
      Capslock-K: up
      Capslock-L: right
```

### Emit the virtual modifier

It's possible to emit `Capslock` from `keymap`, even though the physical press is filtered out.
It just needs a new key combo, like `Capslock-A`:

```yml
virtual_modifiers:
  - Capslock

keymap:
  - remap:
      Capslock-A: Capslock
```

### Restoring the normal function

It's possible emit an action (e.g. `esc`), when the virtual modifier is released alone (i.e. without being interrupted by another key):

```yml
modmap:
  - remap:
      Capslock:
        alone: esc
        held: Capslock
        # Ignore mouse movement when using --mouse
        interruptable:
          not: [XRIGHTCURSOR, XLEFTCURSOR, XDOWNCURSOR, XUPCURSOR]

virtual_modifiers:
  - Capslock

keymap:
  - remap:
      Capslock-H: left
      # etc
```

Note: It can be troublesome when `esc` isn't emitted before it is released.

It's also possible to emit the virtual modifier key itself, but it's more complicated:

```yml
modmap:
  - remap:
      Capslock:
        # Emitted when physically released alone.
        alone: BTN_TRIGGER_HAPPY1
        held: Capslock
        # Ignore mouse movement when using --mouse
        interruptable:
          not: [XRIGHTCURSOR, XLEFTCURSOR, XDOWNCURSOR, XUPCURSOR]

virtual_modifiers:
  - Capslock

keymap:
  - remap:
      # Turn the pseudo key into a real Capslock.
      BTN_TRIGGER_HAPPY1: Capslock
      Capslock-H: left
      # etc
```

Note: When using a key with repeat-action like `tab` as virtual modifier. Then does the repeat action not work.
