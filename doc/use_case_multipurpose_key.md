# Multi-purpose key

### Multi-purpose key with alone_timeout_millis

To make `capslock` also work as `esc`, if it's pressed and released within a timeout:

```yml
modmap:
  - remap:
      Capslock:
        held: Capslock
        alone: esc
        alone_timeout_millis: 200 # Optional, defaults to 1000
```

It works like this:

- If the key is pressed and released within `alone_timeout_millis` without other keys being pressed, it's considered `alone`.
- If another key is pressed before timeout, it's considered `held`.
- If the timeout is reached without other things happening, it's considered `held`.

The alone-action is emitted as press and release right away. The held-action will emit press when it's triggered and
wait to release until the trigger key is released.

### Multi-purpose key with free hold

To use `space` as `shift` when it's held down, but remain `space` if it's not interrupted by another key:

```yml
modmap:
  - remap:
      Space:
        held: Shift_L
        alone: Space
        free_hold: true # Optional, defaults to false.
```

There's no timeout in this case (i.e. `alone_timeout_millis` is ignored).

- The `held` action is triggered when another key is pressed while the multi-purpose key is being held down.
- If the key is released without others key being pressed, it triggers the `alone` action.

This allows a key to be held indefinitely without triggering its `held` state, which is ideal for keys that also serve as modifiers. In this case, you make the `Space` key act as `Shift` when held and another key is pressed, but still type a regular `Space` when tapped.

A drawback of this configuration is, that `space` can't be used for repeating spaces when held down, because that now has new meaning.

Another drawback is that fast writing (e.g. `a`, `space`, `l`) can emit `aL`. One has to release `space` before typing `l` to get `a l`. This can be fixed with `hold_threshold_millis`.

`free_hold` is logically the same as having an infinite `alone_timeout_millis`.

### Multi-purpose key with hold_threshold_millis

```yml
modmap:
  - remap:
      Space:
        held: Shift_L
        alone: Space
        hold_threshold_millis: 200
        free_hold: true
```

This will emit the alone-action, `space`, when it's interrupted by another key before
timeout of `hold_threshold_millis`. This allows `space` to function normally when typing fast. And only after
the timeout will it work as `shift`.

### Multi-purpose key with `interruptable`

You may not want tapping a multi-purpose key to always be interrupted by all types of input events.
You may also have problems tapping multi-purpose keys if you're pressing a lot of keys at once.

You can control which keys can interrupt the `alone` press of a multi-purpose key using the
`interruptable` field:

```yml
modmap:
  - remap:
      Ctrl_L:
        held: Ctrl_L
        alone: Backspace
        interruptable:
          # Ignore mouse movement when using --mouse
          # This is the default
          not: [XRIGHTCURSOR, XLEFTCURSOR, XDOWNCURSOR, XUPCURSOR]
      Alt_L:
        held: Alt_L
        alone: Space
        alone_timeout_millis: 200
        interruptable:
          # Only allow alt+tab to interrupt tapping alt
          only: Tab
```

Input events that would interrupt the `alone` press of these multi-purpose keys will be handled as
normal but without interrupting the key press.

You can set `interruptable: false` to completely disable interruption. Or let all keys interrupt
by setting `interruptable: true`, which also lets mouse wheel and mouse movement interrupt.
