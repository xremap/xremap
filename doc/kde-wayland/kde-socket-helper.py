#!/usr/bin/env python3
"""
KDE socket helper for xremap.
Provides window information via Unix socket to avoid D-Bus EXTERNAL authentication issues.
Requires KDE Plasma 6.
"""

import argparse
import json
import logging
import os
import re
import signal
import socket
import sys
import tempfile
import time
from pathlib import Path
from typing import Optional, Any

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

# KWin script for Plasma 6
KWIN_SCRIPT_CODE = '''
// Plasma 6 active window notifier for xremap

function notifyActiveWindow(client) {
    if (!client) {
        return;
    }

    try {
        callDBus(
            "com.k0kubun.XremapHelper",
            "/com/k0kubun/XremapHelper",
            "com.k0kubun.XremapHelper",
            "NotifyActiveWindow",
            client.caption || "",
            client.resourceClass || "",
            client.resourceName || ""
        );
    } catch (e) {
        print("xremap-helper: Error in notifyActiveWindow: " + e);
    }
}

if (workspace.activeWindow) {
    notifyActiveWindow(workspace.activeWindow);
}

workspace.windowActivated.connect(notifyActiveWindow);

print("xremap-helper: KWin script loaded successfully");
'''

SCRIPT_NAME = 'xremap-helper'


class DBusHelper:
    """Helper class for D-Bus operations."""
    
    def __init__(self):
        self._bus: Optional[dbus.SessionBus] = None
    
    @property
    def bus(self) -> dbus.SessionBus:
        if self._bus is None:
            self._bus = dbus.SessionBus()
        return self._bus
    
    def reset_bus(self):
        """Reset the bus connection to get a fresh one."""
        self._bus = None
    
    def is_service_available(self, service: str = 'org.kde.KWin') -> bool:
        """Check if a D-Bus service is available."""
        try:
            self.bus.get_object(service, '/Scripting')
            return True
        except dbus.exceptions.DBusException:
            return False
    
    def wait_for_service(self, service: str = 'org.kde.KWin', 
                         max_retries: int = 30, retry_delay: float = 2.0) -> bool:
        """Wait for a D-Bus service to become available."""
        for attempt in range(1, max_retries + 1):
            if self.is_service_available(service):
                log.info(f"KWin service is available (attempt {attempt})")
                return True
            
            if attempt < max_retries:
                log.info(f"Waiting for KWin... (attempt {attempt}/{max_retries})")
                time.sleep(retry_delay)
                # Reset bus to get fresh state
                self.reset_bus()
        
        log.error(f"KWin service not available after {max_retries} attempts")
        return False
    
    def call(self, object_path: str, interface: str, method_name: str,
             signature: str = '', args: tuple = (), 
             service: str = 'org.kde.KWin') -> Any:
        """Make a D-Bus call with explicit signature."""
        return self.bus.call_blocking(
            service, object_path, interface, method_name, signature, args
        )
    
    def call_kwin_scripting(self, method_name: str, signature: str = '', 
                            args: tuple = ()) -> Any:
        """Call a method on the KWin Scripting interface."""
        return self.call('/Scripting', 'org.kde.kwin.Scripting', 
                        method_name, signature, args)
    
    def call_script(self, object_path: str, method_name: str, 
                    signature: str = '', args: tuple = ()) -> Any:
        """Call a method on a KWin Script interface."""
        return self.call(object_path, 'org.kde.kwin.Script', 
                        method_name, signature, args)
    
    def introspect(self, object_path: str) -> str:
        """Introspect a D-Bus object and return XML."""
        obj = self.bus.get_object('org.kde.KWin', object_path)
        iface = dbus.Interface(obj, 'org.freedesktop.DBus.Introspectable')
        return iface.Introspect()
    
    def get_child_nodes(self, object_path: str) -> list[str]:
        """Get child node names from introspection."""
        try:
            xml = self.introspect(object_path)
            return re.findall(r'<node name="([^"]+)"', xml)
        except dbus.exceptions.DBusException:
            return []


class KdeSocketHelper:
    def __init__(self, socket_path: str):
        self.socket_path = Path(socket_path)
        self.socket_dir = self.socket_path.parent
        self.server_socket: Optional[socket.socket] = None
        self.running = False
        self.clients: list[socket.socket] = []
        self.current_window: dict = {'caption': '', 'class': '', 'app_id': ''}
        self.kwin_script_path: Optional[str] = None
        self.kwin_script_id: Optional[int] = None
        self.dbus = DBusHelper()

    def setup(self):
        """Create socket directory and socket."""
        self.socket_dir = self._get_socket_directory()
        self.socket_path = self.socket_dir / "kde.sock"
        
        self._remove_existing_socket()
        self._create_server_socket()
        self._setup_socket_watcher()

    def _get_socket_directory(self) -> Path:
        """Determine and create the appropriate socket directory."""
        primary_dir = Path("/run/xremap")
        fallback_dir = Path(f"/run/user/{os.getuid()}/xremap")
        
        if self._try_setup_directory(primary_dir):
            return primary_dir
        
        self._try_setup_directory(fallback_dir, must_succeed=True)
        log.warning(f"Using fallback directory {fallback_dir}")
        return fallback_dir

    def _try_setup_directory(self, directory: Path, must_succeed: bool = False) -> bool:
        """Try to set up a directory, return True on success."""
        try:
            if not directory.exists():
                directory.mkdir(parents=True, exist_ok=True)
                os.chmod(str(directory), 0o755)
                log.info(f"Created directory {directory}")
            elif os.access(str(directory), os.W_OK):
                log.info(f"Using existing directory {directory}")
            else:
                return False
            return True
        except PermissionError:
            if must_succeed:
                raise
            return False

    def _remove_existing_socket(self):
        """Remove existing socket file if present."""
        if self.socket_path.exists():
            self.socket_path.unlink()

    def _create_server_socket(self):
        """Create and configure the Unix server socket."""
        self.server_socket = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.server_socket.bind(str(self.socket_path))
        os.chmod(str(self.socket_path), 0o666)
        self.server_socket.listen(5)
        self.server_socket.setblocking(False)
        
        log.info(f"Socket created at {self.socket_path} (mode 666)")
        log.info(f"Listening on {self.socket_path}")
        print(f"SOCKET_PATH={self.socket_path}")

    def _setup_socket_watcher(self):
        """Set up GLib watch for incoming connections."""
        GLib.io_add_watch(
            self.server_socket.fileno(),
            GLib.IO_IN,
            self._handle_connection
        )

    def _handle_connection(self, fd, condition) -> bool:
        """Handle new client connection."""
        if condition & GLib.IO_IN:
            try:
                client_socket, _ = self.server_socket.accept()
                client_socket.setblocking(False)
                self.clients.append(client_socket)
                log.info(f"Client connected (total: {len(self.clients)})")
                self.broadcast_window_info()
            except BlockingIOError:
                pass
        return True

    def notify_active_window(self, caption: str, res_class: str, res_name: str):
        """Update current window info and broadcast to clients."""
        self.current_window = {
            'caption': caption,
            'class': res_class,
            'app_id': res_name
        }
        self.broadcast_window_info()

    def broadcast_window_info(self):
        """Send window info to all connected clients."""
        if not self.clients:
            return

        message = json.dumps(self.current_window, ensure_ascii=False) + '\n'
        message_bytes = message.encode('utf-8')
        
        self.clients = [
            client for client in self.clients 
            if self._send_to_client(client, message_bytes)
        ]

    def _send_to_client(self, client: socket.socket, data: bytes) -> bool:
        """Send data to a client, return True if successful."""
        try:
            client.sendall(data)
            return True
        except (BrokenPipeError, ConnectionResetError, OSError) as e:
            log.debug(f"Removing disconnected client: {e}")
            self._close_client(client)
            return False

    def _close_client(self, client: socket.socket):
        """Safely close a client socket."""
        try:
            client.close()
        except OSError:
            pass

    def _unload_existing_script(self) -> bool:
        """Try to unload any existing xremap-helper script."""
        try:
            self.dbus.call_kwin_scripting('unloadScript', 's', (SCRIPT_NAME,))
            log.info(f"Unloaded existing {SCRIPT_NAME} script")
            time.sleep(0.5)
            return True
        except dbus.exceptions.DBusException as e:
            log.debug(f"No existing script to unload: {e}")
            return False

    def _write_script_file(self) -> str:
        """Write the KWin script to a temp file and return the path."""
        with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
            f.write(KWIN_SCRIPT_CODE)
            log.info(f"KWin script written to: {f.name}")
            return f.name

    def _load_script(self) -> Optional[int]:
        """Load the script into KWin, return script ID or None."""
        # Try single-argument version first (just the path)
        try:
            script_id = self.dbus.call_kwin_scripting(
                'loadScript', 's', (self.kwin_script_path,)
            )
            log.debug("loadScript succeeded with single argument (path only)")
            return int(script_id)
        except dbus.exceptions.DBusException as e:
            log.debug(f"Single-argument loadScript failed: {e}")
        
        # Try two-argument version (path + name)
        try:
            script_id = self.dbus.call_kwin_scripting(
                'loadScript', 'ss', (self.kwin_script_path, SCRIPT_NAME)
            )
            log.debug("loadScript succeeded with two arguments (path + name)")
            return int(script_id)
        except dbus.exceptions.DBusException as e:
            log.debug(f"Two-argument loadScript failed: {e}")
        
        log.error("All loadScript attempts failed")
        return None

    def _try_run_script(self, script_id: int) -> bool:
        """Try to run script at the expected Plasma 6 path."""
        path = f'/Scripting/Script{script_id}'
        try:
            self.dbus.call_script(path, 'run')
            log.info(f"KWin script started successfully at {path}")
            return True
        except dbus.exceptions.DBusException as e:
            log.debug(f"Failed to start script at {path}: {e}")
            return False

    def _try_run_script_via_introspection(self) -> bool:
        """Try to find and run script by introspecting available nodes."""
        log.warning("Standard path failed, trying introspection...")
        
        nodes = self.dbus.get_child_nodes('/Scripting')
        log.info(f"Found scripting child nodes: {nodes}")
        
        for node in nodes:
            if 'Script' in node or node.isdigit():
                path = f'/Scripting/{node}'
                try:
                    self.dbus.call_script(path, 'run')
                    log.info(f"KWin script started at {path}")
                    return True
                except dbus.exceptions.DBusException as e:
                    log.debug(f"Failed at {path}: {e}")
        
        return False

    def load_kwin_script(self, max_retries: int = 5, retry_delay: int = 2) -> bool:
        """Load KWin script using D-Bus, with retries for boot-time availability."""
        self.kwin_script_path = self._write_script_file()

        # First, wait for KWin to be available (important at boot time)
        log.info("Waiting for KWin to become available...")
        if not self.dbus.wait_for_service('org.kde.KWin', max_retries=30, retry_delay=2.0):
            log.error("KWin never became available")
            return False

        for attempt in range(1, max_retries + 1):
            try:
                # Unload existing script and load new one
                self._unload_existing_script()
                script_id = self._load_script()

                if script_id is None:
                    raise RuntimeError("loadScript returned None")

                log.info(f"Script ID from D-Bus: {script_id}")

                if script_id < 0:
                    raise RuntimeError(f"Script load failed with ID {script_id}")

                self.kwin_script_id = script_id

                # Try to run the script
                if self._try_run_script(script_id):
                    return True
                
                # Fallback to introspection
                if self._try_run_script_via_introspection():
                    return True

                raise RuntimeError("Failed to start KWin script")

            except Exception as e:
                if attempt < max_retries:
                    log.warning(f"Attempt {attempt}/{max_retries}: {e}")
                    log.info(f"Retrying in {retry_delay} seconds...")
                    time.sleep(retry_delay)
                else:
                    log.error(f"Failed to load KWin script after {max_retries} attempts: {e}")
                    return False

        return False

    def cleanup_kwin_script(self):
        """Unload KWin script and clean up temp file."""
        if self.kwin_script_id is not None:
            try:
                self.dbus.call_kwin_scripting('unloadScript', 's', (SCRIPT_NAME,))
                log.info("KWin script unloaded")
            except Exception as e:
                log.error(f"Failed to unload KWin script: {e}")

        if self.kwin_script_path and os.path.exists(self.kwin_script_path):
            try:
                os.unlink(self.kwin_script_path)
            except OSError:
                pass

    def cleanup(self):
        """Clean up all resources."""
        for client in self.clients:
            self._close_client(client)
        self.clients.clear()

        if self.server_socket:
            self.server_socket.close()

        if self.socket_path.exists():
            self.socket_path.unlink()
            log.info(f"Removed socket {self.socket_path}")

        self.cleanup_kwin_script()

    def run(self):
        """Main loop (GLib event loop)."""
        self.running = True
        log.info("Starting KDE socket helper...")

        if not self.load_kwin_script():
            log.error("Could not load KWin script, continuing anyway...")

        loop = GLib.MainLoop()

        def signal_handler(signum):
            log.info(f"Received signal {signum}, shutting down...")
            self.running = False
            self.cleanup()
            loop.quit()

        GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGTERM, signal_handler, signal.SIGTERM)
        GLib.unix_signal_add(GLib.PRIORITY_HIGH, signal.SIGINT, signal_handler, signal.SIGINT)

        loop.run()


class DBusService(Object):
    """D-Bus service that KWin scripts can call."""

    def __init__(self, helper: KdeSocketHelper):
        from dbus.mainloop.glib import DBusGMainLoop
        DBusGMainLoop(set_as_default=True)

        bus_name = BusName("com.k0kubun.XremapHelper", bus=dbus.SessionBus())
        Object.__init__(self, bus_name, "/com/k0kubun/XremapHelper")
        self.helper = helper

    @method("com.k0kubun.XremapHelper")
    def NotifyActiveWindow(self, caption: str, res_class: str, res_name: str):
        """Notify about active window change - called by KWin script."""
        log.debug(f"Received: caption='{caption}', class='{res_class}', name='{res_name}'")
        self.helper.notify_active_window(caption, res_class, res_name)


def main():
    parser = argparse.ArgumentParser(description='KDE Plasma 6 socket helper for xremap')
    parser.add_argument(
        '--socket-path',
        default='/run/xremap/kde.sock',
        help='Path to Unix socket (default: /run/xremap/kde.sock)'
    )
    args = parser.parse_args()

    helper = KdeSocketHelper(args.socket_path)
    helper.setup()

    log.info("Registered D-Bus service com.k0kubun.XremapHelper")
    DBusService(helper)

    helper.run()


if __name__ == '__main__':
    main()
