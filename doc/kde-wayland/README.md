# Xremap Setup Guide for KDE Plasma Wayland

This guide explains how to securely run xremap on KDE Plasma Wayland using a dedicated system user and socket-based helper for application-specific key remapping.

## Table of Contents

1. [Overview and Security Model](#overview-and-security-model)
2. [Architecture](#architecture)
3. [Prerequisites](#prerequisites)
4. [Installation](#installation)
5. [Configuration](#configuration)
6. [Verification](#verification)
7. [Troubleshooting](#troubleshooting)
8. [Cleanup and Removal](#cleanup-and-removal)

---

## Overview and Security Model

### The Security Problem

When xremap needs to remap keyboard input, it requires access to input devices (`/dev/input/event*`). On Linux, this access is controlled by the `input` group. The security concern is:

- **Dangerous approach**: Add your regular user (e.g., `yourusername`) to the `input` group
  - **Problem**: *Any application* running as your user can now read keyboard input
  - **Consequence**: Malicious software, compromised apps, or even buggy programs could log your keystrokes (passwords, chat messages, etc.)

### The Secure Solution

Instead, xremap should run as a **dedicated system user**:

```
┌─────────────────────────────────────────────────────────────┐
│   Your User (`yourusername`)                                │
│   - Runs your applications (browser, terminal, IDE, etc.)   │
│   - NOT in the `input` group                                │
│   - Cannot access raw keyboard input                        │
│   - Even if compromised, cannot read keystrokes             │
└─────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────┐
│   `xremap` System User                                     │
│   - Runs ONLY the xremap process                           │
│   - IS in the `input` group                                │
│   - Has access to input devices (for remapping)            │
│   - Minimal privileges, no login shell                     │
└────────────────────────────────────────────────────────────┘
```

### Benefits

- **Principle of least privilege**: Only xremap can read keyboard input
- **Isolation**: If your user account is compromised, keystrokes remain protected
- **Containment**: Compromised xremap process can only affect key remapping, not your user data
- **Application-specific remapping**: Still works via socket-based helper

### Cross-User Challenge

KDE Plasma's window manager (KWin) runs as your regular user and communicates via D-Bus. For xremap to support application-specific remapping (e.g., different keybindings in Konsole vs Firefox), it needs window information from KWin.

The solution: A **socket-based helper** that runs as your user, talks to KWin via D-Bus, and forwards window information to xremap via a Unix socket.

---

## Architecture

### Component Overview

```
┌───────────────────────────────────────────────────────────────┐
│                    xremap (system user)                       │
├───────────────────────────────────────────────────────────────┤
│  - Reads input devices (/dev/input/eventX)                    │
│  - Processes keystrokes locally (never shares keystroke data) │
│  - Writes remapped output to uinput device                    │
│  - Reads window info from socket (for app-specific remapping) │
│  - Groups: xremap, input                                      │
└───────────────────────────────────────────────────────────────┘
                      ↓ Unix socket (JSON protocol)
┌───────────────────────────────────────────────────────────────┐
│           kde-socket-helper (runs as yourusername)            │
├───────────────────────────────────────────────────────────────┤
│  - Listens on /run/xremap/kde.sock                            │
│  - Loads KWin script via D-Bus (same user)                    │
│  - Receives window change events from KWin                    │
│  - Writes JSON window info to socket for xremap               │
│  - Groups: yourusername, wheel, docker                        │
└───────────────────────────────────────────────────────────────┘
                      ↓ D-Bus session bus (same user)
┌───────────────────────────────────────────────────────────────┐
│                     KWin Window Manager                       │
│                     Runs within KDE session                   │
├───────────────────────────────────────────────────────────────┤
│  - Detects active window changes                              │
│  - KWin script calls helper via D-Bus                         │
│  - Script ID: xremap-helper                                   │
└───────────────────────────────────────────────────────────────┘
```

### Data Flow

**Window/Application Information (for filtering):**
```
User switches window → KWin detects change → KWin script notifies helper via D-Bus
→ Helper writes JSON to socket → xremap reads JSON → xremap updates key remapping
```

**Keystroke Remapping (isolated):**
```
Keyboard → Input device (/dev/input/eventX) → xremap → uinput device
→ Kernel delivers to applications
```

**Critical Security Point:** Keystrokes never flow through the socket or D-Bus. Only window metadata (title, class, name) is shared.

### Communication Protocol

**Helper → xremap** (Unix socket): JSON object on each line
```json
{"caption": "Window Title", "class": "org.kde.konsole", "app_id": "konsole"}
```

**Why JSON?**
- Handles special characters (quotes, pipes, commas) in window titles
- No parsing ambiguity
- Extensible for future features

---

## Prerequisites

### System Requirements

- Fedora Linux with KDE Plasma Wayland
- Python 3 with D-Bus support
- System administrator access (sudo)
- Rust toolchain (for building xremap)

### Packages to Install

```bash
# Required for KDE socket helper
sudo dnf install -y python3-dbus python3-gobject

# Required for building xremap
sudo dnf install -y cargo rust

# Required for uinput device access
# Note: MODE="0660" ensures consistent group-based permissions without ACL interference
echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/input.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

### About tmpfiles.d

The setup uses systemd's `tmpfiles.d` mechanism to create the `/run/xremap` directory at boot time. This directory is used for socket communication between the helper (running as your user) and xremap (running as the xremap user).

The configuration file `xremap-tmpfiles.conf` contains:
```
# Create xremap socket directory
# Owner: root, Group: root, Mode: 1777 (world-writable with sticky bit)
# Sticky bit prevents users from deleting each other's files
d /run/xremap 1777 root root -
```

This configuration ensures:
- `/run/xremap` is created automatically at boot
- Directory has permissions 1777 (world-writable with sticky bit)
- Prevents permission issues between the two users
- Sticky bit prevents users from deleting each other's files (similar to `/tmp`)

This is a one-time setup step - no manual intervention is needed on login or reboot.

---

## Installation

Follow these steps to set up xremap with the secure architecture on KDE Plasma Wayland.

### Step 1: Create the xremap System User

Create a dedicated system user with no home directory or login shell:

```bash
# Create system user
sudo useradd -r -s /usr/sbin/nologin xremap

# Add xremap to input group (required for keyboard access)
sudo usermod -a -G input xremap

# Verify creation
id xremap
```

### Step 2: Build and Install xremap

Build xremap with KDE support and install it system-wide:

```bash
# Navigate to xremap source directory
cd /path/to/xremap

# Build with KDE feature
cargo build --release --features kde

# Install binary
sudo cp target/release/xremap /usr/bin/xremap
sudo chmod +x /usr/bin/xremap

# Verify installation
/usr/bin/xremap --version
```

### Step 3: Install the KDE Socket Helper and tmpfiles.d Config

Copy the Python helper script and tmpfiles.d configuration:

```bash
# Copy helper script
sudo cp kde-socket-helper.py /usr/local/bin/kde-socket-helper

# Make executable
sudo chmod +x /usr/local/bin/kde-socket-helper

# Verify script is accessible
which kde-socket-helper

# Copy tmpfiles.d config for socket directory
sudo cp xremap-tmpfiles.conf /etc/tmpfiles.d/
sudo systemd-tmpfiles --create /etc/tmpfiles.d/xremap-tmpfiles.conf
```

### Step 4: Configure systemd Services

You need two systemd services:
1. **xremap.service**: System service that runs xremap as the xremap user
2. **kde-socket-helper.service**: User service that runs the helper as your regular user

#### Install xremap System Service

```bash
# Copy system service file
sudo cp xremap.service /etc/systemd/system/
```

The service file should look like:
```ini
[Unit]
Description=Xremap (run as xremap user)
After=default.target yourusername-user@yourusername.service kde-socket-helper.service systemd-udev-trigger.service systemd-udev-settle.service

[Service]
Type=simple
User=xremap
Group=input
ExecStart=/usr/bin/xremap --redact --watch=config /etc/xremap/config.yml
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
Environment=RUST_LOG=warn
Environment=KDE_SOCKET=/run/xremap/kde.sock
SupplementaryGroups=input

[Install]
WantedBy=default.target
```

**Note:** Adjust `yourusername` in `After=` directive if your username is different.

#### Install KDE Helper User Service

```bash
# Create user service directory if it doesn't exist
mkdir -p ~/.config/systemd/user

# Copy helper service file
cp kde-socket-helper.service ~/.config/systemd/user/

# Reload user systemd daemon
systemctl --user daemon-reload

# Enable helper to start on login
systemctl --user enable kde-socket-helper.service
```

### Files to Install

Here's a quick reference of all files that need to be installed and where they go:

| Source File | Destination | Description |
|-------------|--------------|-------------|
| `target/release/xremap` | `/usr/bin/xremap` | xremap binary |
| `kde-socket-helper.py` | `/usr/local/bin/kde-socket-helper` | KDE socket helper script (includes retry logic for KWin) |
| `xremap.service` | `/etc/systemd/system/xremap.service` | xremap system service |
| `kde-socket-helper.service` | `~/.config/systemd/user/kde-socket-helper.service` | KDE helper user service |
| `xremap-tmpfiles.conf` | `/etc/tmpfiles.d/xremap-tmpfiles.conf` | tmpfiles.d config for socket directory |
| `/etc/xremap/config.yml` | `/etc/xremap/config.yml` | xremap configuration file (create this) |

### Step 5: Create xremap Configuration Directory and Config

```bash
# Create configuration directory
sudo mkdir -p /etc/xremap

# Set permissions
sudo chown xremap:xremap /etc/xremap
sudo chmod 755 /etc/xremap
```

Create or copy your xremap configuration:

```bash
# Example configuration with application-specific remapping
sudo tee /etc/xremap/config.yml > /dev/null << 'EOF'
# Application-specific remapping
keymap:
  - name: Konsole-specific bindings
    application:
      only: org.kde.konsole
    remap:
      Super-v: C-Shift-v  # Paste in Konsole only

  - name: Copy shortcut (all apps except Konsole)
    application:
      not: org.kde.konsole
    remap:
      Super-c: C-c  # Copy in all apps except Konsole
EOF

# Set ownership
sudo chown xremap:xremap /etc/xremap/config.yml
sudo chmod 644 /etc/xremap/config.yml
```

### Step 6: Start the Services

Start the services in the correct order:

```bash
# Start KDE helper first (must be running before xremap)
systemctl --user start kde-socket-helper.service

# Verify helper is running
systemctl --user status kde-socket-helper.service

# Start xremap
sudo systemctl start xremap.service

# Enable xremap to start on boot
sudo systemctl enable xremap.service

# Verify xremap is running
sudo systemctl status xremap.service
```

### Step 7: Verify Installation

Check that everything is working correctly:

```bash
# Check if socket exists
ls -la /run/xremap/kde.sock
# Should show: srw-rw-rw-. 1 yourusername yourusername ... kde.sock

# Check that xremap user can access socket
sudo -u xremap ls -la /run/xremap/kde.sock
# Should show the socket file

# Check xremap logs for successful connection
journalctl -u xremap -n 50 --no-pager
# Switch between supported apps in your configuration first, then you:
# Should see: application-client: KDE (supported: true)
# Should see: active window: caption: '...', class: '...', name: '...'

# Check helper logs
journalctl --user -u kde-socket-helper -n 50 --no-pager
# Should show: KWin script loaded successfully
# Should show: Using existing directory /run/xremap or Socket created at /run/xremap
```

### Step 8: Test Application-Specific Remapping

Open different applications and verify keybindings work as configured:

```bash
# Test in Konsole
# Open Konsole and press Super-v (should paste with Ctrl-Shift-v)

# Test in VSCodium
# Open VSCodium and press Super-Shift-e (should go to end of line)

# Test in other app (e.g., Firefox)
# Press Super-c (should copy with Ctrl-c)
```

Monitor logs while testing:
```bash
sudo journalctl -u xremap -f --no-pager
# Should see window change messages when switching apps
```

---

## Configuration

### Application-Specific Remapping

xremap can apply different keybindings based on the active application. To find application class names:

1. Check xremap logs when switching windows:
   ```bash
   sudo journalctl -u xremap -f --no-pager
   ```
2. Look for messages like: `active window: caption: '...', class: 'org.kde.konsole', name: '...'`
3. Use the `class` field in your config

Example configuration:
```yaml
keymap:
  - name: Emacs bindings in specific apps
    application:
      only: [codium, emacs, vim]
    remap:
      C-b: left
      C-f: right
      C-p: up
      C-n: down
```

### Window-Specific Remapping

You can also match based on window title (caption):

```yaml
keymap:
  - name: Special keys in password manager window
    window:
      only: /.*Bitwarden.*/
    remap:
      Super-l: C-l  # Lock field
```

### Mode-Based Remapping

For modal interfaces like Vim or Emacs:

```yaml
modmap:
  - name: Emacs-like Ctrl key as modifier
    mode: emacs-mode
    remap:
      Ctrl_L:
        held: Ctrl_L
        alone: Esc
        free_hold: true

keymap:
  - name: Enter Emacs mode
    remap:
      Super-e: { set_mode: emacs-mode }

  - name: Exit Emacs mode
    mode: emacs-mode
    remap:
      Esc: { set_mode: default }
```

---

## Verification

### Security Verification

Verify that your regular user is **not** in the input group:

```bash
groups
# Output should NOT include 'input'
```

Verify that xremap IS in the input group:

```bash
groups xremap
# Output should include 'input'
```

Verify that your user cannot read keyboard input:

```bash
# Try to read from an input device
cat /dev/input/event0
# Should show: Permission denied
```

Verify that xremap can read keyboard input:

```bash
sudo -u xremap ls -la /dev/input/event*
# Should show readable input devices
```

Verify that uinput device has correct permissions:

```bash
ls -la /dev/uinput
# Should show: crw-rw----. 1 root input
# If you see different permissions, re-run the udev rule setup from Prerequisites
```

### Functionality Verification

**Test 1: Global key remapping works**
```bash
# Press CapsLock - should behave as Esc
# Works in all applications
```

**Test 2: Application-specific remapping works**
```bash
# Open Konsole, press Super-v - should paste (Ctrl-Shift-v)
# Open Firefox, press Super-v - should do nothing
```

**Test 3: Window information is received**
```bash
sudo journalctl -u xremap -n 20 --no-pager | grep "active window"
# Should show window info for each window switch
```

**Test 4: KWin script is loaded**
```bash
qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.isScriptLoaded xremap-helper
# Should return: true
```

### Service Status Verification

```bash
# Check xremap system service
sudo systemctl status xremap.service
# Should show: Active: active (running)

# Check KDE helper user service
systemctl --user status kde-socket-helper.service
# Should show: Active: active (running)

# Check socket exists
ls -la /run/xremap/kde.sock
# Should show: srw-rw-rw-. 1 yourusername yourusername
```

---

## Troubleshooting

### xremap shows "application-client: KDE (supported: false)"

**Symptom:** Application-specific remapping doesn't work.

**Possible causes:**

1. **KDE helper not running**
   ```bash
   systemctl --user status kde-socket-helper.service
   ```
   If not running: `systemctl --user start kde-socket-helper.service`

2. **Socket doesn't exist**
   ```bash
   ls -la /run/xremap/kde.sock
   ```
   If missing: Restart helper service

3. **Socket permissions wrong**
   ```bash
   ls -la /run/xremap/kde.sock
   ```
   Should show: `srw-rw-rw-` (mode 666)

4. **xremap can't access socket**
   ```bash
   sudo -u xremap ls -la /run/xremap/kde.sock
   ```
   If permission denied: Check `/run/xremap` directory exists and has correct permissions (1777)

### "Could not connect to kwin-script" Error

**Symptom:** xremap logs show connection error.

**Solution:** This error is normal with socket-based architecture. xremap doesn't need direct D-Bus connection anymore. Verify:
- KDE helper is running
- Socket exists and is accessible
- xremap shows `supported: true` eventually

### Application filtering not working

**Symptom:** Keybindings work the same in all applications.

**Troubleshooting:**

1. Check application class names in logs:
   ```bash
   sudo journalctl -u xremap -f --no-pager
   # Switch windows and look for: active window: class: '...'
   ```

2. Verify config matches class name exactly:
   - Konsole: `org.kde.konsole`
   - VSCodium: `codium`
   - Firefox: `firefox`
   - Chrome: `google-chrome`

3. Test with exact match:
   ```yaml
   keymap:
     - name: Test
       application:
         only: konsole  # Use exact class name
       remap:
         Super-x: left
   ```

### xremap won't start

**Symptom:** Service fails to start.

**Check logs:**
```bash
sudo journalctl -u xremap -n 50 --no-pager
```

**Common issues:**

1. **Config file error**: Syntax error in `/etc/xremap/config.yml`
   - Validate YAML syntax
   - Check for indentation errors

2. **Missing input device access**:
   ```bash
   sudo -u xremap ls -la /dev/input/
   ```
   - Verify xremap is in input group
   - Check udev rules are loaded

3. **Permission denied on config**:
   ```bash
   ls -la /etc/xremap/config.yml
   ```
   - Should be owned by xremap:xremap
   - Should be readable (644)

4. **uinput permission denied**:
   ```
   Error: Failed to prepare an output device: Permission denied (os error 13)
   ```
   This error occurs when the udev rule uses `TAG+="uaccess"` instead of `MODE="0660"`. The uaccess tag sets up ACLs for the logged-in user that can interfere with group-based access.

   **Fix**: Update the udev rule:
   ```bash
   echo 'KERNEL=="uinput", GROUP="input", MODE="0660"' | sudo tee /etc/udev/rules.d/input.rules
   sudo udevadm control --reload-rules
   sudo udevadm trigger
   sudo systemctl restart xremap.service
   ```

   **Verify**: Check that uinput has correct permissions:
   ```bash
   ls -la /dev/uinput
   # Should show: crw-rw----. 1 root input
   ```

### KDE helper won't start

**Symptom:** User service fails to start.

**Check logs:**
```bash
journalctl --user -u kde-socket-helper -n 50 --no-pager
```

**Common issues:**

1. **Missing Python dependencies**:
   ```bash
   sudo dnf install python3-dbus python3-gobject
   ```

2. **DBus session not ready**:
   - Ensure user is logged into KDE session
   - Check `echo $DBUS_SESSION_BUS_ADDRESS`

3. **KWin script fails to load at boot**:
   - Symptom: Helper logs show `ERROR: qdbus loadScript failed` after reboot
   - Cause: KWin window manager may not be ready when helper starts at boot
   - Solution: The helper now includes automatic retry logic (up to 5 attempts, 2 second delay each). If you see this error:
     - Wait a few seconds - the helper will retry automatically
     - Check logs for subsequent success messages: `INFO: Loaded KWin script with ID:`
     - If retries still fail, check helper logs and verify KWin is running
   - Manual test: Try loading the script manually once KWin is running:
     ```bash
     qdbus org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript /path/to/kde-helper-script.js
     ```

### After reboot, services don't start automatically

**Symptom:** Services need manual start after reboot.

**Solutions:**

1. **Verify services are enabled**:
   ```bash
   sudo systemctl is-enabled xremap.service
   systemctl --user is-enabled kde-socket-helper.service
   ```
   Should both return `enabled`.

2. **Check dependencies in service file**:
   ```bash
   sudo cat /etc/systemd/system/xremap.service | grep After=
   ```
   Ensure `After=` includes user service.

3. **Check for startup failures**:
   ```bash
   sudo journalctl -b | grep xremap
   journalctl -b --user | grep kde-socket-helper
   ```

### Socket directory doesn't exist after reboot

**Symptom:** `/run/xremap/` missing after reboot.

**Solution:** The tmpfiles.d config should create it automatically. If it's missing:
```bash
# Verify tmpfiles.d config exists
ls /etc/tmpfiles.d/xremap-tmpfiles.conf

# Manually create directory
sudo systemd-tmpfiles --create /etc/tmpfiles.d/xremap-tmpfiles.conf

# Restart helper service
systemctl --user restart kde-socket-helper.service
```

If the problem persists, verify the tmpfiles.d configuration:
```bash
# Check tmpfiles.d config
sudo cat /etc/tmpfiles.d/xremap-tmpfiles.conf
# Should contain: d /run/xremap 1777 root root -
```

---

## Cleanup and Removal

To completely remove xremap setup:

### Stop and Disable Services

```bash
# Stop xremap system service
sudo systemctl stop xremap.service
sudo systemctl disable xremap.service

# Stop KDE helper user service
systemctl --user stop kde-socket-helper.service
systemctl --user disable kde-socket-helper.service
```

### Remove Service Files

```bash
sudo rm /etc/systemd/system/xremap.service
rm ~/.config/systemd/user/kde-socket-helper.service

# Reload systemd
sudo systemctl daemon-reload
systemctl --user daemon-reload
```

### Remove Executables

```bash
sudo rm /usr/bin/xremap
sudo rm /usr/local/bin/kde-socket-helper
```

### Remove Configuration

```bash
sudo rm -rf /etc/xremap
```

### Remove Socket Directory

```bash
sudo rm -rf /run/xremap
sudo rm /etc/tmpfiles.d/xremap-tmpfiles.conf
```

### Remove xremap User

```bash
sudo userdel xremap
```

### Verify Removal

```bash
# Check user doesn't exist
id xremap
# Should show: no such user

# Check files removed
ls /usr/bin/xremap
# Should show: No such file or directory

ls /etc/xremap
# Should show: No such file or directory
```

---

## Summary

This setup provides a secure, isolated environment for xremap on KDE Plasma Wayland:

- **Security**: Your regular user is not in the `input` group, preventing arbitrary applications from reading keystrokes
- **Isolation**: xremap runs as a dedicated system user with minimal privileges
- **Functionality**: Application-specific and window-specific remapping work via socket-based helper
- **Maintainability**: Simple architecture using standard Unix sockets, systemd services, and tmpfiles.d for automatic directory creation
- **Cross-user socket**: Uses `/run/xremap/` (mode 1777) for secure communication between helper and xremap users

For questions or issues, refer to the troubleshooting section or check the project documentation.
