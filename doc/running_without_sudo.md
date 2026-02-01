# Running xremap without sudo

1. Your normal user should be able to use `evdev` and `uinput`. `evdev` is the mechanism to work
   with input devices. And `uinput` is the mechanism to make artificial input devices.
2. If you want to use application-specific remappings on GNOME Wayland you need to follow the instructions at the bottom.

After following the instructions below, run this command:

```
xremap config.yml
```

When you stop the command everything will go back to normal.

### Pro

- If you launch programs from xremap they will run as your normal user.

### Con

- You have opened your user account up to keylogging. Which is a security risk, that you must consider.
- The remapping remains active when the computer is locked.
- The remapping is also active for other users that are logged in at the same time. If they activate keymappings, that launch programs, they will launch as your user.

## Setup input and output permissions

### Input permission

Add your user to the `input` group:

```bash
sudo gpasswd -a YOUR_USER input
```

### Output permission

Give members of the input group permission to make output devices:

```bash
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess", MODE:="0660", OPTIONS+="static_node=uinput"' | sudo tee /etc/udev/rules.d/99-input.rules
```

There has been other recommendations for this setting, see below if the above doesn't work.

### Module to output artificial events

Check whether `uinput` module is loaded:

```bash
ls -l /dev/uinput
```

If it shows up empty, load the module automatically:

```bash
echo uinput | sudo tee /etc/modules-load.d/uinput.conf
```

### Reboot or try to run right away without reboot

Reboot is likely the only way to make changes take effect.

But you can try to make it work right away, by loading uinput module:

```bash
sudo modprobe uinput
```

Reload permissions of input/output devices:

```bash
sudo udevadm control --reload-rules && sudo udevadm trigger
```

Run in a subshell, where you will have your new groups available:

```bash
su $USER
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
and run:

```sh
echo 'KERNEL=="event*", NAME="input/%k", MODE="660", GROUP="input"' | sudo tee /etc/udev/rules.d/input.rules
```

If you do this, in some environments, `--watch` may fail to recognize new devices due to temporary permission issues.
Using `sudo` might be more useful in such cases.

### Alternatives for uinput module permissions

Note: A reboot is necessary to test whether it works.

In addition to the configuration recommended above are two other possibilities that work on most platforms:

```bash
echo 'KERNEL=="uinput", GROUP="input", TAG+="uaccess"' | sudo tee /etc/udev/rules.d/99-input.rules
```

```bash
echo 'KERNEL=="uinput", RUN+="/usr/bin/setfacl -m g:input:rw /dev/%k"' | sudo tee /etc/udev/rules.d/99-input.rules
```

## Application-specific remappings

### GNOME Wayland

Install xremap's GNOME Shell extension from [this link](https://extensions.gnome.org/extension/5060/xremap/),
switching OFF to ON.

### Other desktop environments

Supported desktops should work without extra work.
