## Multi-purpose key

A multi-purpoase key behaves differently depending on whether it's held or tapped.
There are two ways to think of the functionality: tap-preferred and hold-preferred.
The key difference between tap-preferred and hold-preferred is what happens when the multi-purpose key
is interrupted by another key.

The multi-purpose key starts out in the tap-preferred state, which means it will take the tap-action
right away if it's interrupted by another key, (i.e. it prefers tap when interrupted). After `held_threshold_millis`
it goes into the hold-preferred state, where it will emit the hold-action if it's interrupted. And finally
at `tap_timeout_millis` it will emit the hold-action.

The default `held_threshold_millis` is set to 0. Meaning the multi-purpose key is hold-preferred by default.
The parameters `tap_timeout_millis` and `held_threshold_millis` denote time since the multi-purpose key was pressed.

### In more detail

- Tap-preferred from `0ms` to `held_threshold_millis`:
  - If interrupted by another key press → tap-action
    - The tap-action is pressed then released before the interupting key is pressed.
  - If released alone → tap-action

- Hold-preferred from `held_threshold_millis` to `tap_timeout_millis`:
  - If interrupted by another key press → hold-action
    - The hold-action is pressed before the interupting key is pressed,
      and the hold-action is released when the multi-purpose key is released, independent of the interupting key.
  - If released alone → tap-action

- Always-hold from `tap_timeout_millis` to `∞`:
  - At `tap_timeout_millis` the hold-action is pressed and it's released when the multi-purpose key is released.

The press event that triggers the multi-purpose key is not emitted.

Before a decision is made:

- Repeat events from the trigger key is is not emitted.

When emitting tap-action:

- If the tap-action consists of multiple keys, they are all pressed and then all released in the same order.
- Repeat events from the multi-purpose key is suppressed, and nothing happens when it's released.

The meaning of interrupted is when another key press is emitted from the `modmap`, this means
a physical key can be pressed without interupting a multi-purpose key, as long as it's 'squashed' by
a remap in the `modmap`.

Repeat and release events do not
interrupt the multi-purpose key. They could come from other keys pressed before the multi-purpose key.

What happens in `keymap` doesn't matter as remapping there is performed after
multi-purpose keys are handled.

Multi-purpose keys can interrupt each other. But that's complicated, and is it good or bad?

### Example: Tap-preferred

This configuration is like zmk's tap-preferred hold-tap and qmk's default tap-hold. Except the
tap-action is emitted right away if interrupted before timeout, while it's buffered until timeout
in the two other tools.

```yml
modmap:
  - remap:
      A:
        tap: A
        hold: Shift_l
        hold_threshold_millis: 200
        tap_timeout_millis: 200
```

Tap is preferred for `200ms`, then `shift_l` is pressed and hold.

### Example: Hold-preferred

This configuration is like zmk's hold-preferred hold-tap and qmk's "hold on other key press".

```yml
modmap:
  - remap:
      capslock:
        tap: esc
        hold: Shift_l
        hold_threshold_millis: 0 # This can be omitted, as 0 is the default value.
        tap_timeout_millis: 200
```
