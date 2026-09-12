# Running as a system service

Ensure xremap is installed in `/usr/bin/xremap`, or use the right path below.

Ensure module for creating output devices is loaded. See instructions elsewhere.

These instructions are for `v0.15.13` and later. [See instructions for earlier versions](https://github.com/xremap/xremap/blob/7e6649e442ca445b781e4cf0e90c165f86e717db/doc/running_as_system_service.md). For
users that previously used two binaries for socket and bridge, remember to set `--desktop=socket` in the
system service, as the socket feature can't be auto selected.

### Pro

- It's the most secure way to run `xremap`.
- `xremap` is started automatically when your computer boots, and restarts xremap if it fails.

### Con

- A drawback is that the same config file is used for all users.
- The config file is inconvenient to modify because it's owned by the xremap user.
- If you launch programs from xremap they will run as the `xremap` user. Not your own normal user. Except if you use the `socket` feature, see below.
- If you want to use application-specific remappings it's only possible with the `socket` feature.

## Socket feature (Optional)

The instructions below tell you how to setup with and without the socket feature. With it, the system service will connect to another instance of xremap, which runs as your normal user.

If you don't want this, it doesn't matter which variant of xremap you choose to install.

If you want it choose the full variant of xremap, which includes the socket feature and the ability to connect to all desktops. On the [Releases page](https://github.com/xremap/xremap/releases) choose a version with a name like `xremap-linux-x86_64-full.zip`.

## Create a system user named xremap

```sh
sudo useradd --no-create-home --shell /bin/false --user-group --groups input --system xremap
```

Note: The `xremap` user should only be used for this one purpose, to preserve security separation.
Do not add your own user to the `xremap` group either for the same reason.

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

## Create groups for socket feature (Optional)

The `socket` and `full` variant of xremap lets you use application-specific remappings.

```sh
# Add a group for each user that will use xremap.
sudo groupadd --system xremap-username1
sudo groupadd --system xremap-username2

# Add each user to its respective group
sudo usermod --append --groups xremap-username1 username1
sudo usermod --append --groups xremap-username2 username2
```

Note: You will have to restart for the new groups to take effect.

Note: xremap will be enabled for all users, even those that don't have a `xremap-username` group, but application-specific remapping will only work for users with a corresponding `xremap-username` group.

## Create service file

Create a service file here: `/etc/systemd/system/xremap.service`

```ini
[Unit]
Description=Xremap
After=default.target

[Service]
ExecStart=/usr/bin/xremap --desktop=socket --watch=device /etc/xremap/config.yml
Restart=always
StandardOutput=journal
StandardError=journal
User=xremap
Group=xremap
SupplementaryGroups=input
RuntimeDirectory=xremap
RuntimeDirectoryMode=0755
RuntimeDirectoryPreserve=yes
Environment=RUST_LOG=warn # The default logging level

# Uncomment the following lines for the socket variant.
# Remember to enter the right username and their corresponding uid.
# To get the uid of the current user run "id" in the terminal, and `sudo -u username id` for other users.
#ExecStartPre=install --directory --mode 2770 --owner xremap --group xremap-username1 /run/xremap/uid
#SupplementaryGroups=xremap-username1
#ExecStartPre=install --directory --mode 2770 --owner xremap --group xremap-username2 /run/xremap/uid
#SupplementaryGroups=xremap-username2

[Install]
WantedBy=default.target
```

Adapt the arguments to `xremap` in `ExecStart`. At least consider if you want `--desktop=none` or `--desktop=socket`.

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

## Application-specific remappings (socket feature)

### GNOME

The GNOME extension serves as the second instance of xremap.

#### Install GNOME extension or login again

Install GNOME extension version 12, or later.

In case the GNOME extension is started before the service, you must login again for the GNOME
extension to work.

The GNOME extension is configured to use `/run/xremap/{uid}/xremap.sock` by default. The socket path can be changed with environment variables: set `XREMAP_SOCKET` in _xremap.service_. For the GNOME extension, set `XREMAP_GNOME_SOCKET` in `~/.config/environment.d/99-xremap.conf` or `/etc/environment.d/90-xremap.conf`.

### Other desktops than GNOME

Ensure you are using xremap v0.15.13 or later.

The second instance of xremap connects to your desktop environment. Start it by:

```sh
xremap --bridge
```

The only arguments that `xremap` can take in bridge-mode are:

```sh
xremap --no-window-logging \
       --allow-launch="true or false" \
       --desktop="kde,gnome, ... ,auto or none" \
       --bridge
```

You can run the bridge as a user service or autostart file, see this as inspiration [Running as a user service](running_as_user_service.md).

The bridge only supports the default socket path: `/run/xremap/{uid}/xremap.sock`.

### How the socket feature works

When the `xremap.service` starts the `socket` variant of xremap it will function as the following:

`xremap.service` creates a folder for the chosen users, e.g. `/run/xremap/1000`. This folder is only accessible
to `xremap.service` and that user. When the user starts the GNOME extension or the bridge will the
socket be created in the right folder, and `xremap.service` can connect to this socket.

`xremap.service` monitors the active user to make sure
it gets information from the right user, and launches commands as the right
user (i.e. the user that controls the input devices).

Note: The service must be started at least once since system boot to create the folders.
