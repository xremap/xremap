# Design choices

## Thoughts on reload_config action

There're fundamental problems in replacing just the configuration in a running instance of `xremap`.

1. The information from the old config is scattered out many places, and a software design that
   allows replacing them at runtime, is considerable work to make correct. And it's fragile.
2. The events before the reload-action should be remapped according to old config, while the
   events after should be remapped according to the new config. But making such a barrier is hard
   even to define:
   1. `modmap` can turn one event into two: `A: [{action: reload_config}, B]`. What is `keymap`
      supposed to do here. Is `B` mapped according to old or new config in keymap. New does not makes
      sense, because it's already mapped according to old. Old does not makes sense because it
      comes after reload_config.
   2. Input events that cross the reload barrier: `A:1 time reload_config time A:0`. Mapping `A:1`
      according to old and `A:0` according to new is inconsistent. A solution could be to release
      all keys on output_device (if VirtualDevice supported it). That would work if `A` isn't remapped
      by xremap. But release events can be remapped.

The conclusion: reload_config is fine for simple use cases. But fragile for
complex remapping and/or flow typing. It might be possible to define it consistently,
but that requires a use case that needs it.
