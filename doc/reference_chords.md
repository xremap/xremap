## Chords

### Experimental

Experimental means this feature is likely to change in the feature as it's improved. This
can break configuration files in any version update of xremap.

Features available in `modmap` like: `application`, `window`, `device`, `mode` doesn't work in `experimental_map`.

### Example

When `f11` and `f12` are pressed at the same time, then is `A` pressed:

```yml
experimental_map:
  - chords:
      - keys: [f11, f12]
        actions: A
        timeout: 30 # Optional. Default 30 ms.
```

More examples below.

### Description

Chords are keys that are pressed at the same time (like on a piano).
The keys have to be pressed within the timeout, and if not the phisical
keys are emitted as normal.

While all the trigger keys are held down will the action be repeated. When one of the trigger
keys are released then will the action be released.

This feature works independently of other keys pressed, like modifiers, in other words
modifiers can be held down when pressing a chord.

All key events from the press of the first trigger key is buffered until the decision is made.
This ensures that the action is logically emitted at the press of the first trigger key.
If the action is taken then will trigger keys be fully suppressed, they will neither be pressed
nor released.

The output from `experimental_map` goes to the `modmap` and then goes to `keymap`.

### Possible actions

It's possible to emit nothing, a single key or more keys:

```yml
actions: null       # Emit nothing
actions: A          # A single key
actions: [A, B, C]  # Several keys
```

With several keys in the action, the keys are pressed in the specified order, and they are
all held/repeated until a trigger key is released. Then they are released in the same order they were
pressed.

### Remapping normal keys

It's possible to remap any key using this feature (incl. mouse buttons), but some are better suited than others.
Keys that are used for typing will usually be pressed with little time in between, and
that looks like simultaneous if they are within the timeout. For those keys a small
timeout like `10ms` might be needed.

A drawback of a small timeout is that pressing the keys simultaneously can be difficult.

Example: Press shift by simultaneously pressing `S` and `D` with the left hand:

```yml
experimental_map:
  - chords:
      - keys: [S, D]
        actions: LeftShift
        timeout: 10 # To allow for fast typing
      - keys: [K, L]
        actions: LeftShift
        timeout: 10 # To allow for fast typing
```

There are sequences of letters that are more likely in normal typing, e.g. `odt` is common
in `good`, `food`, etc. Finding uncommon sequences is a good way to avoid misactivation.

## Limitations

Some keyboards have limitations on simultaneous keys. It can limit the total number of
simultaneous keys, or it can limit some combinations of keys, so the keyboard simply emits
nothing, when they are pressed simultaneously.

## Examples

### Example: Modifier chords

Using modifiers in chords reduces the risk of triggering them accidentally. Pressing
normal modifier combinations must just be done a little slower.

Pressing `LeftCtrl` and `K` simultaneously will emit `A`.

```yml
experimental_map:
  - chords:
      - keys: [LeftCtrl, K]
        actions: A
        timeout: 50 # Timeout can be higher, when modifiers are in the keys.
```

### Example: Modifier action

It's possible to emit modifiers from a chord. This allows pressing keyboard combos like `ctrl-v`.

```yml
experimental_map:
  - chords:
      - keys: [f11, f12]
        actions: [LeftCtrl, V]
```
