#!/usr/bin/env python3
"""
KDE socket helper for xremap.
Provides window information via Unix socket to avoid D-Bus EXTERNAL authentication issues.
"""

import argparse
import json
import logging
import os
import signal
import socket
import sys
import tempfile
from pathlib import Path
from typing import Optional

import dbus
import dbus.mainloop.glib
from dbus.service import Object, method, BusName
from gi.repository import GLib

# Set up logging
logging.basicConfig(
    level=logging.INFO,
    format='%(levelname)s: %(message)s',
    stream=sys.stdout
)
log = logging.getLogger('xremap-kde')


class KdeSocketHelper:
    def __init__(self, socket_path: str):
        self.socket_path = Path(socket_path)
        self.socket_dir = self.socket_path.parent
        self.server_socket: Optional[socket.socket] = None
        self.running = False
        self.clients: list[socket.socket] = []
        self.current_window: dict = {
            'caption': '',
            'class': '',
            'app_id': ''
        }
        self.kwin_script_path = None
        self.kwin_script_id = None

    def setup(self):
        """Create socket directory and socket"""
        import pwd

        # Use system-wide runtime directory for cross-user access
        self.socket_dir = Path("/run/xremap")

        # Only try to create /run/xremap if it doesn't exist
        if not self.socket_dir.exists():
            try:
                self.socket_dir.mkdir(parents=True, exist_ok=True)
                # Set permissions to allow xremap user to access
                os.chmod(str(self.socket_dir), 0o755)
                log.info(f"Created directory {self.socket_dir}")
            except PermissionError:
                # If we can't create /run/xremap (as user), fall back to user runtime
                uid = os.getuid()
                self.socket_dir = Path(f"/run/user/{uid}/xremap")
                self.socket_dir.mkdir(parents=True, exist_ok=True)
                os.chmod(str(self.socket_dir), 0o755)
                log.warning(f"Could not create /run/xremap, using {self.socket_dir} instead")
        else:
            # Directory exists, verify we can write to it
            if os.access(str(self.socket_dir), os.W_OK):
                log.info(f"Using existing directory {self.socket_dir}")
            else:
                # Can't write to /run/xremap, fall back to user runtime
                uid = os.getuid()
                self.socket_dir = Path(f"/run/user/{uid}/xremap")
                self.socket_dir.mkdir(parents=True, exist_ok=True)
                os.chmod(str(self.socket_dir), 0o755)
                log.warning(f"Cannot write to /run/xremap, using {self.socket_dir} instead")

        self.socket_path = self.socket_dir / "kde.sock"

        # Remove existing socket if present
        if self.socket_path.exists():
            self.socket_path.unlink()

        # Create Unix socket
        self.server_socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.server_socket.bind(str(self.socket_path))

        # Set permissions to allow xremap user to connect
        os.chmod(str(self.socket_path), 0o666)
        log.info(f"Socket created at {self.socket_path} (mode 666 for universal access)")

        # Listen for connections
        self.server_socket.listen(5)
        self.server_socket.setblocking(False)
        log.info(f"Listening on {self.socket_path}")
        # Print socket path for easy reference
        print(f"SOCKET_PATH={self.socket_path}")

        # Setup GLib watch for incoming connections
        GLib.io_add_watch(
            self.server_socket.fileno(),
            GLib.IO_IN,
            self.handle_connection
        )

    def handle_connection(self, fd, condition):
        """Handle new connection"""
        if condition & GLib.IO_IN:
            try:
                client_socket, _ = self.server_socket.accept()
                client_socket.setblocking(False)
                self.clients.append(client_socket)
                log.info(f"Client connected (total: {len(self.clients)})")

                # Send current window info immediately
                self.broadcast_window_info()
            except BlockingIOError:
                pass
        return True

    def notify_active_window(self, caption: str, res_class: str, res_name: str):
        """Called by KWin script via D-Bus"""
        self.current_window = {
            'caption': caption,
            'class': res_class,
            'app_id': res_name
        }
        self.broadcast_window_info()

    def broadcast_window_info(self):
        """Send window info to all connected clients"""
        if not self.clients:
            return

        # Use JSON format for robustness (handles special characters, quotes, etc.)
        data = json.dumps(self.current_window, ensure_ascii=False)
        message = f"{data}\n"

        # Send to all clients, removing any that fail
        remaining_clients = []
        for client in self.clients:
            try:
                client.sendall(message.encode('utf-8'))
                remaining_clients.append(client)
            except (BrokenPipeError, ConnectionResetError, OSError) as e:
                log.debug(f"Removing disconnected client: {e}")
                try:
                    client.close()
                except:
                    pass

        self.clients = remaining_clients

    def load_kwin_script(self, max_retries=5, retry_delay=2):
        """Load KWin script using qdbus, with retries for boot-time availability"""
        import subprocess
        import time

        script_code = '''function notifyActiveWindow(client) {
    if (!client) {
        return;
    }

    try {
        callDBus(
            "com.k0kubun.XremapHelper",
            "/com/k0kubun/XremapHelper",
            "com.k0kubun.XremapHelper",
            "NotifyActiveWindow",
            "caption" in client ? client.caption : "",
            "resourceClass" in client ? client.resourceClass : "",
            "resourceName" in client ? client.resourceName : ""
        );
    } catch (e) {
        print("Error in notifyActiveWindow: " + e);
    }
}

// Call for existing active window - with error handling
try {
    if (workspace.activeClient) {
        callDBus(
            "com.k0kubun.XremapHelper",
            "/com/k0kubun/XremapHelper",
            "com.k0kubun.XremapHelper",
            "NotifyActiveWindow",
            workspace.activeClient.caption,
            workspace.activeClient.resourceClass,
            workspace.activeClient.resourceName
        );
    }
} catch (e) {
    print("Error calling initial active window: " + e);
}

if (workspace.windowList) {
    workspace.windowActivated.connect(notifyActiveWindow);
} else {
    workspace.clientActivated.connect(notifyActiveWindow);
}

print("KWin script loaded and connected to workspace");
'''

        # Write script to temp file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
            f.write(script_code)
            self.kwin_script_path = f.name
            log.info(f"KWin script written to: {self.kwin_script_path}")

        # Try loading with retries
        for attempt in range(1, max_retries + 1):
            try:
                # Use qdbus to load and start script
                # Load script
                result = subprocess.run(
                    ['qdbus', 'org.kde.KWin', '/Scripting', 'org.kde.kwin.Scripting.loadScript',
                     self.kwin_script_path, 'xremap-helper'],
                    capture_output=True,
                    text=True
                )

                if result.returncode != 0:
                    if attempt < max_retries:
                        log.warning(f"Attempt {attempt}/{max_retries}: qdbus loadScript failed: {result.stderr}")
                        log.info(f"Retrying in {retry_delay} seconds...")
                        time.sleep(retry_delay)
                        continue
                    else:
                        log.error(f"qdbus loadScript failed after {max_retries} attempts: {result.stderr}")
                        return False

                # Script loaded successfully
                script_id_str = result.stdout.strip()
                log.info(f"Raw script ID from qdbus: '{script_id_str}'")

                try:
                    script_id = int(script_id_str)
                except ValueError:
                    log.error(f"Invalid script ID: {script_id_str}")
                    return False

                if script_id < 0:
                    log.error(f"Script load returned negative ID: {script_id} - likely script execution error")
                    # Still try to start it
                    script_id = abs(script_id)

                log.info(f"Loaded KWin script with ID: {script_id}")
                self.kwin_script_id = script_id

                # Start script - try different object paths for KDE5/KDE6
                for obj_path_fn in [lambda i: f'/{i}', lambda i: f'/Scripting/Script{i}']:
                    obj_path = obj_path_fn(self.kwin_script_id)
                    result = subprocess.run(
                        ['qdbus', 'org.kde.KWin', obj_path, 'org.kde.kwin.Script.run'],
                        capture_output=True,
                        text=True
                    )

                    if result.returncode == 0:
                        log.info(f"KWin script started successfully at {obj_path}")
                        if result.stderr:
                            log.debug(f"Script stderr: {result.stderr}")
                        return True
                    else:
                        log.debug(f"Failed to start script at {obj_path}: {result.stderr}")

                log.error("Failed to start KWin script")
                return False

            except Exception as e:
                if attempt < max_retries:
                    log.warning(f"Attempt {attempt}/{max_retries}: Exception during KWin script loading: {e}")
                    log.info(f"Retrying in {retry_delay} seconds...")
                    time.sleep(retry_delay)
                    continue
                else:
                    log.error(f"Failed to load KWin script after {max_retries} attempts: {e}")
                    import traceback
                    log.error(traceback.format_exc())
                    return False

        log.error(f"Failed to load KWin script after {max_retries} attempts")
        return False

    def cleanup_kwin_script(self):
        """Unload KWin script"""
        if not self.kwin_script_id:
            return

        try:
            bus = dbus.SessionBus()
            kwin_scripting = bus.get_object('org.kde.KWin', '/Scripting')
            scripting_interface = dbus.Interface(kwin_scripting, 'org.kde.kwin.Scripting')
            scripting_interface.unloadScript('xremap-helper')
            log.info("KWin script unloaded")
        except Exception as e:
            log.error(f"Failed to unload KWin script: {e}")

        # Clean up temp file
        if self.kwin_script_path and os.path.exists(self.kwin_script_path):
            try:
                os.unlink(self.kwin_script_path)
            except:
                pass

    def cleanup(self):
        """Clean up resources"""
        # Close all client connections
        for client in self.clients:
            try:
                client.close()
            except:
                pass
        self.clients.clear()

        if self.server_socket:
            self.server_socket.close()

        if self.socket_path.exists():
            self.socket_path.unlink()
            log.info(f"Removed socket {self.socket_path}")

        # Cleanup KWin script
        self.cleanup_kwin_script()

    def run(self):
        """Main loop (GLib event loop)"""
        self.running = True

        log.info("Starting KDE socket helper...")

        # Load KWin script
        if not self.load_kwin_script():
            log.error("Could not load KWin script, continuing anyway...")

        # Create GLib main loop
        loop = GLib.MainLoop()

        # Set up signal handlers using GLib (works better with GLib main loop)
        def signal_handler(signum):
            log.info(f"Received signal {signum}, shutting down...")
            self.running = False
            # Cleanup before quitting
            self.cleanup()
            # Quit main loop (not a new instance!)
            loop.quit()

        # Watch for SIGTERM and SIGINT using GLib's signal handling
        # Pass signal number as user_data so handler receives it
        GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGTERM, signal_handler, signal.SIGTERM)
        GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGINT, signal_handler, signal.SIGINT)

        # Run main loop
        loop.run()


class DBusService(Object):
    """D-Bus service that KWin scripts can call"""

    def __init__(self, helper: KdeSocketHelper):
        # Set up D-Bus main loop for GLib
        from dbus.mainloop.glib import DBusGMainLoop
        DBusGMainLoop(set_as_default=True)

        # Create bus name and object
        bus_name = BusName("com.k0kubun.XremapHelper", bus=dbus.SessionBus())
        Object.__init__(
            self,
            bus_name,
            "/com/k0kubun/XremapHelper"
        )
        self.helper = helper

    @method("com.k0kubun.XremapHelper")
    def NotifyActiveWindow(self, caption: str, res_class: str, res_name: str):
        """Notify about active window change - called by KWin script"""
        log.debug(f"Received: caption='{caption}', class='{res_class}', name='{res_name}'")
        self.helper.notify_active_window(caption, res_class, res_name)


def main():
    parser = argparse.ArgumentParser(description='KDE socket helper for xremap')
    parser.add_argument('--socket-path',
                        default='/run/xremap/kde.sock',
                        help='Path to Unix socket (default: /run/xremap/kde.sock)')
    args = parser.parse_args()

    # Create helper instance
    helper = KdeSocketHelper(args.socket_path)

    # Setup socket
    helper.setup()

    # Register D-Bus service
    log.info("Registered D-Bus service com.k0kubun.XremapHelper")
    DBusService(helper)

    # Run main loop
    helper.run()


if __name__ == '__main__':
    main()
