## Key sequence

Remap-action sets a temporary remap to use for matching. Which is cancelled if next event doesn't match anything. And takes the form:

```yml
keymap:
  - remap:
      Ctrl-X:
        timeout_key: space # Optional. Defaults to nothing. Can also be an array.
        timeout_millis: 150 # Optional. No timeout by default.
        remap:
          Ctrl-Q: Esc # Action
```

This is recognized by the nested `remap` keyword. The nested remap takes the same data as the top-level remap.
