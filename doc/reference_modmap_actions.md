## Modmap Actions

`modmap` has different actions from `keymap` and `experimental_map`.

It doesn't support `null` action as `experimental_map` does. No reason it couldn't.

### Key-to-key remap

Actions for key-to-key remaps are pressed when the remap is
triggered, and released when the trigger key is released.

The action has normal repeat behavior. For a key-to-key remap it means, when the
trigger key is pressed and held, its action is repeated.
In the example below, `A` is repeated when `f11` is pressed held down.

Example:

```yml
modmap:
  - remap:
      f11: A # Single key
      f12: [A, B, C] # Zero or more keys
```

With several keys in the action, the keys are pressed in the
specified order, and released in the same order.

### Multi purpose key

The multi purpose key has an action for `tap` and an action for `hold`.
They take the same actions, but obviously differ in the way they press/release the action.

The modifiers in an action will be pressed first and released last. That is only
something that is done for the multi purpose key. This allows emitting a key combo from
the `tap` or `hold` action.

Example:

```yml
modmap:
  - remap:
      CAPSLOCK:
        # held emits the key combo: SHIFT_L-A
        held: [SHIFT_L, A]
        alone: CAPSLOCK
```

### Press/release key

Actions for press/release key works differently from the other two remap types in `modmap`.
It allows the same actions as [keymap does](reference_actions.md).
This also means that the press-action is pressed and released when the remap is triggered.
And when the trigger key is released is the release-action pressed and released.

The press-action, repeat-action and release-action are all emitted directly without further
remapping in keymap. This is different from all other actions in `modmap`, which goes through
keymap for further remapping.

The key that triggered the press/release key does not bypass further remapping in `keymap`,
which is a little counterintuitive.
