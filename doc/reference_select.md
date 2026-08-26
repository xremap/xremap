## Select between operators

`select` chooses exactly one of several operators, that take a small `timeout` to decide if they match.

Think of operators as small programs, that decide how keys are remapped. They take key events
as input and output new key events.

Examples of operators are: [Double tap](reference_double_tap.md), [Chords (simultaneous keys)](reference_chords.md), [Oneshot key](reference_oneshot.md) and [Throttle keys](reference_throttle.md).

### Experimental

Experimental means this feature is likely to change in the future as it's improved. This
can break configuration files in any version update of xremap, and will be noted in CHANGELOG.md.

Features available in `modmap` like: `device`, `mode`, key-to-key mapping,
multi-purpose key and press/release key don't work in `experimental_map`.
But application-specific remapping with `application` and `window` works since `v0.15.5`.

### Description

`select` takes a list of operators and chooses the first in order, that matches. That operator
will get all the events, and the other operators will get no events and have no visible effect.

Operators have the ability to defer decision of whether they match events, the events are buffered
until a decision is made. This makes it possible for fx `double` tap to completely squash the
key that triggered it.

Operators of this type should have a short timeout to make sense. Otherwise would the keyboard
just freeze. But if it's short enough the delay can be tolerated or even unnoticable.

## Examples

### Example: Fast and slow double tap

The following allows two different actions depending on how fast double tap is performed.
The first operator `double` takes precedence. If it doesn't match will the
second `double` get a chance to match. And eventually if it doesn't match after `300ms`
will the event fallthrough to next level, which is `modmap`.

```yml
experimental_map:
  - remap:
      A:
        select:
          - double: B
            timeout: 150
          - double: C
            timeout: 300 # This timeout is also started when the key is pressed.
```

### Example: Double tap and oneshot action otherwise

The following allows an action when double tapping and a oneshot otherwise.
If the double tap doesn't match (default timeout: 200ms), will it look like
it never existed, and the `oneshot` action is emitted.

```yml
experimental_map:
  - remap:
      s_l:
        select:
          - { double: B }
          - { oneshot: s_l }
```

Note: An active oneshot will emit a press of `s_l` to next level when cancelled, so `double` doesn't see it.
So the first `s_l`, in a double tap in that state, will cancel
oneshot and go through, and the next `s_l` will not activate `double`.
Which mean oneshot will activate again. That's very counterintuitive.

The above example is equivalent to the following:

```yml
experimental_map:
  - remap:
      s_l: { double: B }
  - remap:
      s_l: { oneshot: s_l }
```

That's because the second remap has the same `application` and `window` matcher, which is no one.
The purpose of more than one `remap` group is the extra properties on those groups.
Everything in `experimental_map` will eventually just be collected into _one_ long list of things.
