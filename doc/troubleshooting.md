## Troubleshooting

### Get logs from xremap

Run xremap with `RUST_LOG=debug`:

```bash
RUST_LOG=debug xremap config.yml
```

You can choose between the levels: `error`, `warn`, `info`, `debug`.

Beware that `debug` prints all your keyboard strokes, so don't leave it on for normal use, that would be a security risk.

### Get the key names of your keyboard

Start xremap with logging enabled. It will print all key events:

```bash
RUST_LOG=debug xremap config.yml
```

### Get application id/class

[There's instructions here to find those](../#application)

### Remapping works but fails sometimes or in some applications

This might be caused by key events emitted to fast by xremap. You can add a delay:

```yml
keypress_delay_ms: 10
# Rest of you config file
```

`10 ms` is good default value, but try `100 ms` to be certain if this is the cause of your problems.
But don't go larger, because your keyboard will become very slow/unresponsive.

### Application-specific remappings don't work

#### How to check if they work

First check whether a basic configuration works: [print application id/class](../#all-desktops).

If it doesn't work you can see what desktop `xremap` is trying to use in the output, it
should print something like `application-client: GNOME (supported: true)`.

#### Can it work another way

Try to find application id/class another way: [instructions in the same section](../#application).

If this doesn't work you might be using the wrong desktop, the options are [here](../#installation).

#### Do you use sudo

When running as sudo there are extra [installation instructions](running_with_sudo.md).

### Application-specific remappings don't work for GNOME desktop

Check that the extension is installed correctly, by running:

```sh
busctl --user call org.gnome.Shell /com/k0kubun/Xremap com.k0kubun.Xremap WMClasses
```

It should print a list of open windows. If it doesn't the extension is probably not installed correctly.

If you use `sudo` to run xremap, this must work both with and without `sudo`. If only `sudo` doesn't work
try the instructions for sudo/GNOME [here](./running_with_sudo.md#gnome-wayland).

### Application-specific remappings don't work for COSMIC desktop

This might be due to the version you use. Only the latest wayland version of COSMIC
is supported.

For older versions of COSMIC you could try the GNOME extension, but whether it works in unknown.

### Info about your desktop

```sh
echo XDG_SESSION_TYPE=$XDG_SESSION_TYPE
echo XDG_SESSION_DESKTOP=$XDG_SESSION_DESKTOP
echo XDG_CURRENT_DESKTOP=$XDG_CURRENT_DESKTOP
echo WAYLAND_DISPLAY=$WAYLAND_DISPLAY
echo XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR
```
