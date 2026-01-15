# Running xremap without sudo

Installation:

1. Your normal user should be able to use `evdev` and `uinput`.
2. If you want to use application-specific remappings on GNOME Wayland you need to follow the instructions at the bottom.

Run this command, when you stop the command everything will be back to normal.

```
xremap config.yml
```

## Setup input and output permissions

In Ubuntu, this can be configured by running the following commands and rebooting your machine.

```bash
sudo gpasswd -a YOUR_USER input
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/input.rules
```

### Arch Linux

The following can be used on Arch.

```bash
lsmod | grep uinput
```

If this module is not loaded, add to `/etc/modules-load.d/uinput.conf`:

```bash
uinput
```

Then add udev rule.

```bash
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/99-input.rules
```

Then reboot the machine.

### Debian

Make sure `uinput` is loaded same as in Arch:

```
lsmod | grep uinput
```

If it shows up empty:

```bash
echo uinput | sudo tee /etc/modules-load.d/uinput.conf
```

Add your user to the `input` group and add the same udev rule as in Ubuntu:

```bash
sudo gpasswd -a YOUR_USER input
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/input.rules
```

Reboot the machine afterwards or try:

```bash
sudo modprobe uinput
sudo udevadm control --reload-rules && sudo udevadm trigger
```

### NixOS

The following can be used on NixOS.

Ensure `uinput` is enabled in your `configuration.nix`:

```nix
hardware.uinput.enable = true;
boot.kernelModules = [ "uinput" ];
```

Then add the rule to the `udev` extra rules in your `configuration.nix`:

```nix
services.udev.extraRules = ''
  KERNEL=="uinput", GROUP="input", TAG+="uaccess"
  '';
```

The new rule will be added to `/etc/udev/rules.d/99-local.rules`. See [NixOS documentation](https://search.nixos.org/options?channel=24.11&show=services.udev.extraRules&from=0&size=50&sort=relevance&type=packages&query=services.udev) for additional information.

Rebuild with `nixos-rebuild switch`. Note you may also need to reboot your machine.

### Other platforms

In other platforms, you might need to create an `input` group first
and run `echo 'KERNEL=="event*", NAME="input/%k", MODE="660", GROUP="input"' | sudo tee /etc/udev/rules.d/input.rules` as well.

If you do this, in some environments, `--watch` may fail to recognize new devices due to temporary permission issues.
Using `sudo` might be more useful in such cases.

## Application-specific remappings

### GNOME Wayland

Install xremap's GNOME Shell extension from [this link](https://extensions.gnome.org/extension/5060/xremap/),
switching OFF to ON.

### Other desktop environments

These should work without extra work.
