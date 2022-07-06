# What this is

This is a [Nix flake](https://nixos.wiki/wiki/Flakes) that installs and configures [xremap](https://github.com/k0kubun/xremap)

# What is tested in flake

The flake was developed to mostly work in Sway. Thus the following features are tested:

| Scenario | No features | Sway | Gnome | X11 |
| - | - | - | - | - |
| Tested? | :heavy_check_mark: | :heavy_check_mark: | :heavy_multiplication_x: | :heavy_multiplication_x: |

# How to use

Add following to your `flake.nix`:

```nix
{
    inputs.xremap-flake.url = "github:VTimofeenko/xremap?dir=nix-flake";
}
```

And import the `xremap-flake.nixosModules.defalut` module.

Alternatively, flake application can be `nix run` to launch xremap without features.

# Configuration

Following `services.xremap` options are exposed:

* `config` – configuration file for xremap. See [original repo](https://github.com/k0kubun/xremap) for examples.
* `withSway` – whether to enable Sway support
* `withSway` – whether to enable Gnome support
* `package` – which package for xremap to use
* `userId` – user under which Sway IPC socket runs
* `deviceName` – the name of the device to be used. To find out the name, you can check `/proc/bus/input/devices`
* `watch` – whether to watch for new devices
