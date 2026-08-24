## Oneshot a key (aka: sticky key)

A oneshot key emits an action at press. The action is kept until next key is pressed and released just after.

Fx when a oneshot key is pressed and released, then is fx `LeftShift` hold, and it can be used to capitalize the next letter.

Working since v0.15.12

### Experimental

Experimental means this feature is likely to change in the future as it's improved. This
can break configuration files in any version update of xremap, and will be noted in CHANGELOG.md.

Features available in `modmap` like: `device`, `mode`, key-to-key mapping,
multi-purpose key and press/release key don't work in `experimental_map`.
But application-specific remapping with `application` and `window` works since `v0.15.5`.

### Description

The action is emitted at press and kept when the oneshot key is released.
The action remains pressed until next key is pressed and releases immediately after that key.

If on the other hand, the oneshot key is held when the next key is pressed, it functions
completely normal. That is, the action is released when the oneshot key is released.

The third option is that the oneshot key is itself repressed before other keys are pressed.
The action is cancelled in that case.

The oneshot action is not interrupted by mouse movement or scroll. But is interrupted by mouse click if `--mouse` argument is used.

Note: Other modifiers and other oneshot keys also interrupt a oneshot action.
It might not be the intended behavior, and is subject to change, based on user feedback.

### Example: Use shift as oneshot shift

```yml
experimental_map:
  - remap:
      s_l: { oneshot: s_l }
```

`Shift` still functions as a normal modifier.

## References

- https://docs.qmk.fm/one_shot_keys
- https://man.archlinux.org/man/extra/keyd/keyd.1.en#Example_3
