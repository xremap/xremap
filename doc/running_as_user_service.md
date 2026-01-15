# Running as a user service

Ensure you can [run xremap without sudo](running_without_sudo.md), because the user service is
basically just for autostarting xremap.

### Pro

- `xremap` is started automatically when you login, and restarts if it fails.

### Con

- If you launch programs from xremap they will run as your normal user, but it's in a service
  context, which might not work the same as when you run commands normally.
- The same drawbacks as running without sudo applies.

## Setup

Put your config file at `~/.config/xremap/config.yml` and
copy `example/xremap.service` to `~/.config/systemd/user/xremap.service`.

```bash
cp example/xremap.service ~/.config/systemd/user/xremap.service
```

> [!WARNING]
> Make sure `xremap` installation path matches the path in `xremap.service`.

then run

```bash
sudo systemctl --user start xremap.service
```

To start the service on login, `sudo systemctl --user enable xremap.service` may sometimes work.
However, it may fail to recognize the window manager if you start xremap too early.
Consider copying `example/xremap.desktop` to `~/.config/autostart/xremap.desktop` if the platform supports it.

If you change the service file `xremap.service` you need to restart the service:

```sh
sudo systemctl --user daemon-reload
sudo systemctl --user restart xremap.service
```

You can see status and logs with the following commands:

```sh
sudo systemctl --user status xremap.service
sudo journalctl -fu xremap.service
```
