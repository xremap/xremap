# <img src='.github/xremap.png' style='height: 32px; margin-top: 8px; margin-bottom: -4px;' alt='Xremap'> Experimental fork.

[![GitHub Actions](https://github.com/hpccc53/xremap/actions/workflows/build.yml/badge.svg)](https://github.com/hpccc53/xremap/actions/workflows/build.yml)

`xremap` is a key remapper for Linux.

This is a fork for experimenting with new features for xremap. Full compatibility is retained with official xremap.

This experimental fork is compatible with xremap in the following way:

| Version              | Xremap version |
| -------------------- | -------------- |
| 0.0.1 (not released) | 0.14.4         |

## Changelog

Changes made on top of xremap:

**Add all mouse buttons to output device** ([PR 6](https://github.com/hpccc53/xremap/pull/6))

Make it possible to emit all mouse buttons from a config file. Before it was only possible to click some of the mouse buttons.

**Throttle output events** ([PR 5](https://github.com/hpccc53/xremap/pull/5))

Delay (if needed) between:

- press and release of the same key. But not the other way around.
- press of ordinary key and press/release of modifier key.
- press/release of modifier key and press of ordinary key.

Config file:

```yml
throttle_ms: 10 # Defaults to 0
```

Notes

This is useful because some applications and desktops don't handle key events correctly when they are emitted fast. By adding these delays there is time to register combos.

There's a similar configuration `keypress_delay_ms`, but it's only added when emitting key combos from `keymap`, but there are other places, where it's useful.
