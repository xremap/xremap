# Running as user service

Put your config file at `~/.config/xremap/config.yml` and
copy `example/xremap.service` to `~/.config/systemd/user/xremap.service`.

```bash
cp example/xremap.service ~/.config/systemd/user/xremap.service
```

> [!WARNING]
> make sure `xremap` installation path matches `xremap.service` path

then run

```bash
systemctl --user start xremap.service
```

To start the service on boot, `systemctl --user enable xremap.service` may sometimes work.
However, it may fail to recognize the window manager if you start xremap too early.
Consider copying `example/xremap.desktop` to `~/.config/autostart/xremap.desktop` if the platform supports it.

## Drawsback

- The remapping remains active when the computer is locked.
- The remapping is also active for other users, that are logged in at the same time.
