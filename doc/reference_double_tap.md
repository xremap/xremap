## Double tap

### Experimental

Experimental means this feature is likely to change in the future as it's improved. This
can break configuration files in any version update of xremap, and will be noted in CHANGELOG.md.

Features available in `modmap` like: `device`, `mode`, key-to-key mapping,
multi-purpose key and press/release key don't work in `experimental_map`.
But application-specific remapping with `application` and `window` works since `v0.15.5`.

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

The actions emitted by double tap are the same as most remaps
in `experimental_map` [described here](reference_expmap_actions.md).

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

The multi-purpose key doesn't allow repeating the alone-action, because
holding the key emits the hold-action.

A usual trick is to double tap the multi-purpose key to repeat it. `bypass`
makes sure the `space` key goes around `modmap`.

```yml
experimental_map:
  - bypass: true
    remap:
      space:
        double: space

modmap:
  - remap:
      space:
        held: shift_l
        alone: space
```

Works for v0.15.15 [See example prior to that](https://github.com/xremap/xremap/blob/e418175570a6b10d3b39867730802439d2ce6979/doc/reference_double_tap.md#example-double-tap-to-decide-alone-action)
