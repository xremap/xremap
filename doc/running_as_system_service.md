# Running as a system service

Ensure xremap is installed in `/usr/bin/xremap`, or use the right path below.

Ensure module for creating output devices is loaded. See instructions elsewhere.

Other things might be needed, because these instructions are fairly new.

### Pro

- It's the most secure way to run `xremap`.
- `xremap` is started automatically when your computer boots, and restarts if it fails.

### Con

- A drawback is that the same config file is used for all users.
- The config file is inconvenient to modify because it's owned by the xremap user.
- If you launch programs from xremap they will run as the `xremap` user. Not your own normal user, except for GNOME desktop, see below.
- If you want to use application-specific remappings it's only possible on GNOME, see below.

## Create a system user named xremap

```sh
sudo useradd --no-create-home --shell /bin/false --user-group --groups input --system xremap
```

Note: The `xremap` user should only be used for this one purpose, to preserve security separation. Do not add your own user to the
`xremap` group either for the same reason.

## Best security: create xremap-{username} groups

The "socket" feature variant of xremap monitors logind to select the socket path based on the active seated user session. The socket path (`/run/xremap/{uid}/xremap.sock` by default) will be created with permissions that restrict socket access to xremap and the `{uid}` user. Currently this only works with GNOME, although other desktop environments could and should make use of the same socket protocol.

```sh
# Add a xremap-{username} group for each user that will use xremap.
# Replace {usernameN} with the username.
sudo groupadd --system xremap-{username1}
sudo groupadd --system xremap-{username2}
sudo groupadd --system xremap-{username3}

# Add each user to its respective xremap-{username} group
sudo usermod --append --groups xremap-{username1} {username1}
sudo usermod --append --groups xremap-{username2} {username2}
sudo usermod --append --groups xremap-{username3} {username3}
```

Note: xremap will be enabled for all users, even those not having a `xremap-{username}` group, but application- and window-specific mappings will not function for users with no corresponding `xremap-{username}` group.

## Place your config file a central location

Create a folder for configuration:

```sh
sudo mkdir -p /etc/xremap
```

Copy your config file to `/etc/xremap/config.yml`

Change the ownership of the file:

```sh
sudo chown xremap:xremap /etc/xremap/config.yml
sudo chmod 644 /etc/xremap/config.yml
```

## Create service file

Create a service file here: `/etc/systemd/system/xremap.service`

```ini
[Unit]
Description=Xremap
After=default.target

[Service]
ExecStart=/usr/bin/xremap --watch=device /etc/xremap/config.yml
Restart=always
StandardOutput=journal
StandardError=journal
User=xremap
Group=xremap
SupplementaryGroups=input
RuntimeDirectory=xremap
RuntimeDirectoryMode=0755
RuntimeDirectoryPreserve=yes

# Add two lines for each xremap-{username} group created above, replacing
# {username} and {uid} for each. Otherwise, comment or remove these lines.
ExecStartPre=install --directory --mode 2770 --owner xremap --group xremap-{username} /run/xremap/{uid}
SupplementaryGroups=xremap-{username}

[Install]
WantedBy=default.target
```

Adapt the arguments to `xremap` in `ExecStart`.

Start the service

```sh
sudo systemctl start xremap.service
```

If you change the service file `xremap.service` you need to restart the service:

```sh
sudo systemctl daemon-reload
sudo systemctl restart xremap.service
```

You can see status and logs with the following commands:

```sh
sudo systemctl status xremap.service
sudo journalctl -fu xremap.service
```

## Autostart xremap at boot:

Run this command once:

```sh
sudo systemctl enable xremap.service
```

## Application-specific remappings

### GNOME

#### Create special service

Ensure you are using xremap v0.14.10 or later.

The GNOME extension is configured to use `/run/xremap/{uid}/xremap.sock` by default. The socket path can be changed with environment variables: set `XREMAP_SOCKET` in _xremap.service_, and for the GNOME extension, set `XREMAP_GNOME_SOCKET` in `~/.config/environment.d/99-xremap.conf` or `/etc/environment.d/90-xremap.conf`.

#### Install GNOME extension or login again

Install GNOME extension version 12. If you already have this version installed you will need to login again.

Note: The service must be started at least once since system boot to create the folder `/run/xremap`. In
case the GNOME extension is started before the service, you must login again for the GNOME
extension to work.

### Other desktop environments

Work in progress.
