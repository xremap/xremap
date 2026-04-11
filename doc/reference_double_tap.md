## Double tap

### Experimental

Experimental means this feature is likely to change in the feature as it's improved. This
can break configuration files in any version update of xremap.

Features available in `modmap` like: `application`, `window`, `device`, `mode` doesn't work in `experimental_map`.

### Example

Double tap super key (aka win/meta) to mute volume:

```yml
experimental_map:
  - remap:
      LeftMeta:
        double: mute
        timeout: 200 # Optional. Default is 200. Meaning 200ms.
```

Working since v0.14.18

More examples below.

### Description

While the trigger key is held down is the action repeated. When the trigger key
is released then will the action also be released.

This feature works independently of other keys pressed, like modifiers, in other words
modifiers can be held when pressing a chord.

All key events from the press of the trigger key is buffered until the decision is made.
This ensures that the action is logically emitted at the press of the first press.
If the action is taken then will the trigger key be fully suppressed, it will neither be pressed
nor released.

The output from `experimental_map` goes to the `modmap` and then goes to `keymap`.

### Drawbacks

It's possible to remap any key using this feature, but some are better suited than others.

It's not convenient to make a remapping on fx the `O` key, because it will be triggered when
typing `good`.

### Possible actions

It's possible to emit nothing, a single key or more keys:

```yml
double: null       # Emit nothing
double: A          # A single key
double: [A, B, C]  # Several keys
```

With several keys in the action, the keys are pressed in the specified order, and they are
all held/repeated until a trigger key is released. Then they are released in the same order they were
pressed.

## Examples

### Example: Double tap mouse button

```yml
experimental_map:
  - remap:
      btn_right:
        double: R
        timeout: 200 # Optional. Default is 200. Meaning 200ms.
```

To use mouse buttons xremap must be started with the `--mouse` argument.

### Example: Double tap to decide alone-action

The multi-purpose key doesn't currently allow repeating the alone-action, because
holding the key emits the hold-action.

A usual trick is to double tap the multi-purpose key to repeat it:

```yml
experimental_map:
  - remap:
      space:
        # Needed to side-step the remapping below.
        double: BTN_TRIGGER_HAPPY1

modmap:
  - remap:
      BTN_TRIGGER_HAPPY1: space
      space:
        held: shift_l
        alone: space
```
