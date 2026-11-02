## Key sequence

Remap-action sets a temporary remap to use for matching. Which is cancelled if next event doesn't match:

```yml
keymap:
  - remap:
      Ctrl-X:
        timeout_millis: 150 # Optional. No timeout by default.
        timeout: space # Optional. Only relevant when timeout_millis is set.
        remap:
          Ctrl-Q: Esc
```

A key sequence is recognized by the `remap` nested below another `remap`.
Nested remaps remap from key combos to actions, just as the top-level remap.
But nested remaps don't take the options: `mode`, `application`, etc.

On the other hand it takes timeout-properties.

The modifiers can be kept down or repressed for each step in the sequence, so
`Ctrl-X Ctrl-Q` match above as `Ctrl-(XQ)` does.

### Cancel

When a key sequence turns out to not match will it be cancelled.
When a sequence cancels are all keys pressed up to that point ignored, by default.
`timeout_millis` changes this behavior.

The key that didn't match and cancelled the sequence will get a chance to match
a remapping, just as it would have if it didn't cancel a sequence.

Modifiers don't cancel key sequences. Otherwise would `Ctrl` in `Ctrl-Q` cancel
the sequence and not make much sense.

### Timeout

When the option `timeout_millis` is given a value will the key sequence get cancelled after
the configured time. By default will the latest trigger key be tapped at that point in time.
This can be overwritten by setting the `timeout` property.

The timeout only applies to the position in the sequence it's defined at.
At each step in the sequence must a new timeout be set, or there's no timeout.

It would be possible to have a timeout on the first and last key in a sequence, but no
timeout on keys in the middle of the sequence.

Sometimes cancel by non-match emits timeout-actions. User feedback is needed to
determine whether this should always or never happen. Sometimes seems unintended behavior.

#### Possible actions for timeout

The actions emitted by timeout is similar to [what can be defined under the remap property](reference_actions.md).
Support for these actions: set_mode, set_mark, with_mark, escape_next_key are not allowed and still under development.

#### Key emit mode

There are two ways to emit the action: `A`. Key-emit-mode does not take modifiers into
consideration, and just taps the key `A`. Combo-emit-mode will lift all modifiers
not mention in `A`, tap `A` and then restore the modifiers again. As `A` has no modifiers
at all, will all modifiers be lifted.

Timeout-actions are default key-emit-mode as opposed to normal keymap actions,
which are default combo-emit-mode. The default applies to fx `A` where there's
no modifiers defined. If modifiers are specified in the action, e.g. `C-A`, then it's always combo-emit-mode.

### Delayed Match

To better support multiple key sequences, the possible candidates that
can match are tried in turn, and the first that match a full sequence wins.
Candidates are removed if they stop matching the sequence.
So a candidate can only match sequences that are longer than
those that shadow them in preceding candidates.

`C-S-J C-J` will match second. The first will not win just because
it's first and match the first step `C-S-J`.

```yml
keymap:
  - remap:
      S-J:
        remap: { C-K: A }
      C-J:
        remap: { C-J: B }
```

If a remap with non-remap actions match, then will its action
be emitted, and later remaps will be ignored.

`C-S-J` could match both below, but the second is ignored because the first match and
can emit.

```yml
keymap:
  - remap:
      C-J: A
      S-J:
        - remap: { J: B }
```

Note: When considering multiple candidates will actions that mix remaps and non-remaps be ignored.

Note: Delayed match doesn't work well with other features. And will likely see changes in the future.
User feedback is welcome to create a picture of what works, and what doesn't.

### Trigger action

It's possible to emit a default action, when the nested remap is triggered.
When pressing `C-J` is `C-A` emitted in addition to starting matching key sequences:

```yml
keymap:
  - remap:
      C-J:
        - C-A # Select all text, independent of further matches
        - remap:
            C-J: C-C # Copy text
            C-K: delete # Delete text
```

`C-J C-J` selects all text and copies it. `C-J C-K` selects all text and deletes it.

Note: This doesn't work properly with Delayed Match. Because when the first match has mixed
actions, other potential candidates are ignored.

This can be seen with following config where the sequence: `C-S-J C-K` will give a `C`.
But commenting in the trigger action: `A` will make it stop working.

```yml
keymap:
  - remap:
      C-J:
        # - A
        - remap: { C-J: B }
      S-J:
        - remap: { C-K: C }
```

### Inexact match

It's possible to use inexact match of modifiers, so all the sequences:
`J J`, `S-J J`, `J S-J` and `S-J S-J` match below:

```yml
keymap:
  - remap:
      J:
        remap: { J: B }
```

The first two sequences produce `b` the other two produce `B`, because
extra modifiers are preserved in the output.

Note: Inexact doesn't work well together with delayed match, because
adding a remap that can match exactly, but never end up matching
breaks the first remap.

Below will the sequences `S-J S` and `S-J S-J` match the exact remap in first step, and
any inexact candidates are completely ignored. So they will stop working.

```yml
keymap:
  - remap:
      J:
        remap: { J: B }
      S-J:
        remap: { X: Y }
```
