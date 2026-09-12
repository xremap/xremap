## Install from source

### From crates.io

All xremap releases are published to [crates.io](https://crates.io/).
It contains only source code, which makes it the easiest way to compile from source.

First ensure [Rust is installed](https://doc.rust-lang.org/cargo/getting-started/installation.html),
then run one of the following commands:

```bash
cargo install xremap --features full       # All desktops
cargo install xremap --features x11        # X11
cargo install xremap --features gnome      # GNOME Wayland
cargo install xremap --features kde        # KDE-Plasma Wayland
cargo install xremap --features wlroots    # Sway, Wayfire, etc.
cargo install xremap --features hypr       # Hyprland
cargo install xremap --features niri       # Niri
cargo install xremap --features cosmic     # COSMIC Wayland
cargo install xremap --features pantheon   # Pantheon Wayland (aka Secure)
cargo install xremap --features socket     # Variant for system service
cargo install xremap                       # No desktop integration
```

You may also need to install `libx11-dev` to run `xremap` for X11.

You may find a list of supported compositors for wlroots [here](https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1#compositor-support).

#### Run the installed binary

```sh
xremap --version
```

### From GitHub

```sh
git clone https://github.com/xremap/xremap.git
cd xremap
```

#### Run project

```sh
cargo run --features x11 -- config.yml
```

#### Run with debug logs

```sh
RUST_LOG=debug RUST_BACKTRACE=1 cargo run --features x11 -- config.yml
```

#### Run tests

Unit tests:

```sh
cargo test --lib
```

E2e tests. Can cause problems, so more careful:

```sh
cargo test --features device-test
```
