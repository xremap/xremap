# Use modifier to trigger remap

Modifiers can trigger a remap, but they work a little different than a normal key trigering a remap, like: `Ctrl-C`.

After a modifier triggers a remap will it be pressed. So it can still be used as a modifier afterwards. In
other words: physically pressed modifiers will also be logically pressed. This is different from normal keys used to trigger a remap. It will look like they where never pressed.

This feature works best if modifiers on one side of the keyboard are used as modifiers, and modifiers
on the other side of the keyboard are used as triggers.

### Use control keys as Home and End

```yml
keymap:
  - remap:
      Ctrl_L-Ctrl_R: End
      Ctrl_R-Ctrl_L: Home
```

It will emit `Home` or `End` depending on which order the modifiers are pressed.
The match is inexact, so pressing `Shift_R-Ctrl_R-Ctrl_L` will select text to the beginning
of a line, because it emits: `Shift_R-Home`.

### Use two shift keys to toggle Capslock

```yml
keymap:
  - remap:
      Shift_L-Shift_R: Capslock
      Shift_R-Shift_L: Capslock
```

The shift keys act as modifiers for each others.

Note: that toggling Capslock and holding either of the shift keys, will reverse the
meaning of capslock, as that is what shift does. So Capslock must be toggled and the shift
keys released for this remapping to work.
