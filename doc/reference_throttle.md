## Throttle a key

`throttle_ms` controls how fast a key can be reused.

Working since v0.15.12

### Experimental

Experimental means this feature is likely to change in the future as it's improved. This
can break configuration files in any version update of xremap, and will be noted in CHANGELOG.md.

Features available in `modmap` like: `device`, `mode`, key-to-key mapping,
multi-purpose key and press/release key don't work in `experimental_map`.
But application-specific remapping with `application` and `window` works since `v0.15.5`.

### Description

It works by letting the first event-cycle through. That is press-repeat-release are let through.
Then are all events squashed until timeout is reached, where the key can again be used.

Timeout may happen while the key is physically pressed, and in that case will repeat and release events
still go through, so the key is consistently released.

## Examples

### Example: Protect angainst accidental double tap

If you have a tendency to accidentally pressing a key twice, when the intention
was to just press it once, it's possible to set a high timeout, so the key
doesn't have any effect the second time.

```yml
experimental_map:
  - remap:
      kpminus: { throttle_ms: 2000 }
```

### Example: Prevent faulty keyboard events

If a keyboard is faulty and outputs double tap, when it was actually just pressed once.
It's possible to drop the extra events:

```yml
experimental_map:
  - remap:
      kpminus: { throttle_ms: 50 }
```

Note: This is only a partial solution to the problem, because altough double tap
is prevented, the key is released again fast. So it looses its repeat-functionality.
A better solution is a method like QMK, that throttles press and release-events together.

## References

- Definitions used: [throttle](https://rxjs.dev/api/operators/throttle), [debounce](https://rxjs.dev/api/operators/debounce)
- QMK: [Contact bounce / contact chatter](https://docs.qmk.fm/feature_debounce_type)
