## Actions

Remapping lets you assign actions to keys. Actions can only be used in `keymap`
and [press/release keys](reference_press_release_key.md).

### Example: Run programs

Run a program when `KEY_A` is pressed (and repeated). The trigger key is disabled entirely.
The program is just started and ignored, it does not block key processing.

```yml
keymap:
  - remap:
      Capslock: { launch: ["notify-send", "Hello World!"] }
```

Note: Holding down `capslock` will generate repeat events, which will run the program every say `50ms`.

### Example: Run bash script

By using yaml notation for multi-line text it's possible to write scripts:

```yml
keymap:
  - remap:
      Capslock:
        launch:
          - "bash"
          - "-c"
          - |
            NAME=`whoami`
            WORKDIR=`pwd`
            notify-send "Username: $NAME" "Workdir: $WORKDIR"
```

### Example: Key events

It's possible to emit the individual press, repeat and release events:

```yml
throttle_ms: 10 # Slows down events.
keymap:
  - remap:
      Capslock: [{ press: B }, { repeat: B }, { release: B }]
```

Note: `throttle_ms` is necessary for some applications because actions are
sent without any delay in-between by default.

Note: That repeat events are usually ignored by the kernel. It emits the repeat events itself.

### Example: Sleep

Block key processing in the given amount of `ms`.

```yml
keymap:
  - remap:
      Capslock: { sleep: 10 }
```
