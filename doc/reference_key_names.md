## Key names

All keys and modifiers are case-insensitive. And the `KEY_` prefix is optional, though the `BTN_` prefix is required.

### Ordinary keys

#### Aliases for left and right modifiers, special to xremap

- `CONTROL_L`, `CTRL_L`, `C_L`
- `SHIFT_L`, `S_L`
- `ALT_L`, `A_L`, `M_L`
- `SUPER_L`, `WINDOWS_L`, `WIN_L`, `W_L`

The original keys are named: `KEY_LEFTCTRL`, `KEY_LEFTSHIFT`, `KEY_LEFTALT` and `KEY_LEFTMETA`.

#### Numpad keys are named like

- `KEY_KP1`, `KEY_KPDOT`, `KEY_KPASTERISK`

#### Some special keys

- `KEY_ESC` can't be replaced with `KEY_ESCAPE`.
- `KEY_LEFTMETA` means the left windows key (aka super key).
- `KEY_102ND` can be the key placed near the bottom left of the keyboard.
- The print key can have the name `KEY_SYSRQ`, even though `KEY_PRINT` also exists.
- `KEY_SCROLLUP` etc. does likely nothing. See below for scroll functionality.
- `BTN_TRIGGER_HAPPY1`..`BTN_TRIGGER_HAPPY40` can be used as `virtual keys`, that can be emitted
  from `modmap` and remapped in `keymap`. Because they likely has no effect if emitted from xremap.

### Modifiers

When a key combo like: `Ctrl-a` is specified it will match both left and right modifiers. It's the
same as specifying: Both `Ctrl_L-a` and `Ctrl_R-a`.

- `CONTROL`, `CTRL`, `C`
- `SHIFT`, `S`
- `ALT`, `A`, `M`
- `SUPER`, `WINDOWS`, `WIN`, `W`

### Numbers

Numbers must be given with quotation marks, when used like this:

```yml
keymap:
  - remap:
      Capslock: "1"
```

Other positions don't need quotation: like `1: A`, or `Capslock: C-1`

### Mouse buttons

Mouse buttons have the prefix: `BTN_`, which is not optional.
E.g. `BTN_LEFT`, `BTN_RIGHT`, `BTN_MIDDLE`.

#### Scroll

Some special keys exist: `XRIGHTSCROLL`, `XLEFTSCROLL`, `XUPSCROLL`, `XDOWNSCROLL`, they
match the when a mouse scrolls. But they cannot be emitted from xremap.

#### Mouse move

Keys for mouse movement are named: `XRIGHTCURSOR`, `XLEFTCURSOR`, `XDOWNCURSOR` and `XUPCURSOR`.

### Unknown keys

Keys that don't have names can be identified by their key code, e.g. `Code_123`.
