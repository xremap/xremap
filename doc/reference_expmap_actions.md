## Expmap Actions

`experimental_map` has different actions from `modmap` and `keymap`.
Actions in `experimental_map` are generally pressed when the remap is
triggered, and released when the remap is released.

The action also has normal repeat behavior. For double tap it means, when the
trigger key is pressed twice and held, its action is repeated.
In the example below, `A` is repeated when `f11` is pressed twice and held down.

### Key events

It's possible to emit nothing, a single key or more keys:

```yml
experimental_map:
  - remap:
      f10:
        double: null # No action
      f11:
        double: A # Single key
      f12:
        double: [A, B, C] # Zero or more keys
```

The key(s) are all pressed when the remap is triggered and all released when the remap is released.

With several keys in the action, the keys are pressed in the
specified order, and released in the same order.
