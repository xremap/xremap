# Upgrade to v0.15

## Change 1 - Interrupting multipurpose key

Multipurpose keys were previously interrupted by mouse move events by defualt.
Which is quite unexcepted. That is changed, so one has to opt-in to this.
Mouse scroll still interrupts multipurpose keys by default.

## Change 2 - Relative mouse events

This version changes the behavior of 'relative mouse events'. These events
can be remapped even though they are not really keys.
The events are primarily mouse wheel (scroll):

```
XRightScroll, XLeftScroll, XUpScroll, XDownScroll
```

And mouse move:

```
XRightCursor, XLeftCursor, XDownCursor, XUpCursor
```

These events could previously appear in both `modmap` and `keymap` but they
made little sense in `modmap`, so that has been removed. This makes maintaining
and adding new features easier.

# Examples of migrating from modmap to keymap

## Turn mouse events into keys

Scroll up is replaced with `KEY_A`, and mouse right is replaced with `KEY_B`

```yml
modmap:
  - remap:
      XUpScroll: KEY_A
      # Mouse move right
      XRightCursor: KEY_B
```

To upgrade just replace `modmap` with `keymap`.

## Launch command when scrolling

```yml
modmap:
  - remap:
      XDownScroll:
        press: { launch: [bash, -c, "echo hello >> /home/USER_NAME/test_file"] }
        skip_key_event: false
```

There is no replacement for this. Because `keymap` can't emit `XDownScroll` and there isn't
something like `skip_key_event` in `keymap`. Had the config used `skip_key_event: true` however it works in `keymap`.

## Mapped events from modmap is used in keymap

```yml
modmap:
  - remap:
      XUpScroll: C
keymap:
  - remap:
      C: D
```

Can be replaced with:

```yml
keymap:
  - remap:
      XUpScroll: D
```

## Multipurpose key interrupted by mouse move

```yml
modmap:
  - remap:
      Capslock:
        alone: KEY_A
        held: KEY_B
        # This will also make mouse move events interrupt
        interruptable: true
```

## Multipurpose key

This use case has little purpose, because the release is fired immediately.
so the alone definition is always emitted.

```yml
modmap:
  - remap:
      XUpScroll:
        alone: KEY_A
        held: KEY_B
```
