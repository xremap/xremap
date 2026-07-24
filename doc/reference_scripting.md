## Scripting

Supported since: `v0.15.10`.

The configuration file doesn't allow for scripting, it's only a declarative way to define remapping.

But there's an easy way to make `xremap` programmable by using the language that `xremap` is written
in: `Rust`.

It might be a hard learning curve to use `Rust`, but the time spent learning `Rust` might be useful other
places as `Rust` is a popular language.

### Experimental

`xremap` is foremost a program, that is used as is. Being able to program it is an additional feature
and has lower priority, which is why it will not be as stable as the program is, for which breaking changes
are carefully avoided.

Therefore, there can be breaking changes in every version of `xremap` when used like this. Though
it's avoided if possible.

### Warning about scripting

When mapping key events it's easy to cause problems like a stuck/unresponsive keyboard and/or mouse. To avoid
having to force-shutdown the computer there are some advice:

- Have an USB keyboard, that can be plugged/replugged into the computer, when this happens. Xremap will not
  grab that keyboard if it's not started with `--watch`.
- Start `xremap` without the `--mouse` flag. When the keyboard is stuck, the mouse will still be working
  normally, and can be used to close `xremap`. This way `--watch` can still be used.
- Make a kill-key. And have `xremap` shutdown if that key is pressed. If `xremap` doesn't react to that,
  then it itself is stuck, and the USB keyboard method can be used. Even with `--watch` because `xremap`
  is stuck so it won't be able to grab the new input devices.

### Features (aka API)

The things that can be programmed in `Rust` is very limited to avoid unneeded complexity. The
features can be gradually expanded when users report a need for it.

See also: [Complete API reference](https://docs.rs/xremap/latest/xremap/)

#### Remap individual key events

In this function users are given each key event, which are key-presses, key-repeats and key-releases.
And must then return a vector (list) of new key events.

A remap that does nothing looks like this:

```rust
fn on_key_event(&mut self, key: Key, value: KeyValue) -> Vec<(Key, KeyValue)>{
  return vec![(key, value)];
}
```

A remap that disable a key looks like this:

```rust
fn on_key_event(&mut self, key: Key, value: KeyValue) -> Vec<(Key, KeyValue)>{
  if key == KeyCode::KEY_KPMINUS {
      return vec![];
  }
  return vec![(key, value)];
}
```

A remap that turns `Capslock` into `Ctrl` looks like this:

```rust
fn on_key_event(&mut self, key: Key, value: KeyValue) -> Vec<(Key, KeyValue)>{
  if key == KeyCode::KEY_CAPSLOCK {
      return vec![(KeyCode::KEY_LEFTCTRL, value)];
  }
  // Default
  return vec![(key, value)];
}
```

### How to start

See also: [A full example](../example/scripting)

#### Install rust

[Official instructions](https://rust-lang.org/tools/install/)

#### Create a new project with cargo

```sh
mkdir hello-xremap && cd hello-xremap
cargo init
```

#### Add xremap

With no desktop support:

```sh
cargo add xremap
```

With desktop support:

```sh
cargo add xremap --features kde
```

Other desktops can be chosen than `KDE`.

#### Add sample `main.rs`

```rust
use xremap::{KeyCode, KeyValue, Plugin, Result, xremap_cli};

fn main() -> Result<()> {
    xremap_cli(CustomPlugin {})
}

pub struct CustomPlugin {}

impl Plugin for CustomPlugin {
    fn on_key_event(&mut self, key: KeyCode, value: KeyValue) -> Vec<(KeyCode, KeyValue)> {
        if key == KeyCode::KEY_A && value == KeyValue::Press {
            println!("Key A presssed");
        };

        vec![(key, value)]
    }
}
```

### Example project

An example of how to avoid multiple key presses within a certain time. Only if that time has passed since
the key was first pressed can the key be used again.

[Squash key project](../example/scripting)

### Common tasks

#### Run the project

```sh
cargo run config.yml
```

The same command-line arguments can be used as the ordinary xremap-binary.

Add `--` before any command-line arguments:

```sh
cargo run -- --mouse config.yml
```

#### Select a specific version of `xremap`.

```sh
cargo add xremap@0.15.10
```

This also works for updating `xremap`.
