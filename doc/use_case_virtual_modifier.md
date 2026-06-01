# Use normal key as modifier

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

virtual_modifiers:
  - Capslock

keymap:
  - remap:
      Capslock-H: left
      # etc
```

Note:

- It can be troublesome when `esc` isn't emitted before it is released.
- Works for v0.15.0 [See example prior to that](https://github.com/xremap/xremap/blob/f8451418a425fc91451f692d490fc32114b85eb1/doc/use_case_virtual_modifer.md#restoring-the-normal-function)

It's also possible to emit the virtual modifier key itself, but it's more complicated:

```yml
modmap:
  - remap:
      Capslock:
        # Emitted when physically released alone.
        alone: BTN_TRIGGER_HAPPY1
        held: Capslock

virtual_modifiers:
  - Capslock

keymap:
  - remap:
      # Turn the pseudo key into a real Capslock.
      BTN_TRIGGER_HAPPY1: Capslock
      Capslock-H: left
      # etc
```

Note: Works for v0.15.0 [See example prior to that](https://github.com/xremap/xremap/blob/f8451418a425fc91451f692d490fc32114b85eb1/doc/use_case_virtual_modifer.md#restoring-the-normal-function)

### Using a more normal key, like `tab`

Capslock is a little special. It doesn't have a repeat-action, and its
action can be moved from press of `capslock` to release of `capslock` without
loss of convenience.

`tab` is an example of a key with repeat-action, and its action can't be moved from press to release,
without loss of convenience. Fx `alt-tab` will change window when tab is released, which is
inconvenient. `tab` also becomes fragile when `alt-tab` is typed fast, because `alt` can be released before
`tab` is released, which will look like `tab`, because it's `alt:1 alt:0 tab:1 tab:0`.

**Repeating `tab` with key combo**

It's possible to repeat `tab`, by emitting `tab` when pressing a combo, fx `tab-q`.

**Repeating `tab` with double tap**

```yml
experimental_map:
  - remap:
      tab:
        double: BTN_TRIGGER_HAPPY1

modmap:
  - remap:
      tab:
        alone: BTN_TRIGGER_HAPPY1
        held: tab

virtual_modifiers:
  - tab

keymap:
  - remap:
      BTN_TRIGGER_HAPPY1: tab
      tab-H: left
```

Note: Working since v0.14.18

**Convenient `alt-tab`**

Not currently possible in xremap, but is theoretically possible:

1. A timeout could emit a normal `tab`, so it functions normally, just after a timeout.
2. `tab` pressed when other keys are held, will make it function normally, not as virtual
   modifier. This way will `alt-tab` work completely normal, because `alt` is pressed before `tab`.
