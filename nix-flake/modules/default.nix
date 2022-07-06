xremap: naersk-lib: { pkgs, config, ... }:

let
  cfg = config.services.xremap;
  package = (import ../overlay xremap naersk-lib pkgs { inherit (cfg) withSway withGnome withX11; }).xremap-unwrapped;
in
with pkgs.lib;
{
  options.services.xremap = {
    withSway = mkEnableOption "support for Sway";
    withGnome = mkEnableOption "support for Gnome";
    withX11 = mkEnableOption "support for X11";
    package = mkOption {
      type = types.package;
      default = package;
    };
    config = mkOption {
      type = types.either types.str types.path;
      default = "";
      example = ''
                modmap:
                - name: Except Chrome
                application:
        not: Google-chrome
             remap:
        CapsLock: Esc
                  keymap:
                  - name: Emacs binding
                  application:
        only: Slack
              remap:
              C-b: left
              C-f: right
              C-p: up
              C-n: down
      '';
    };
    userId = mkOption {
      type = types.int;
      default = 1000;
      description = "User ID that would run Sway IPC socket";
    };
    deviceName = mkOption {
      type = types.str;
      description = "Device name which xremap will hook into";
    };
    watch = mkEnableOption "running xremap watching new devices";
  };
  config =
    let
      userPath = "/run/user/${toString cfg.userId}";
    in
    {
      environment.etc."xremap_config.yml".text = cfg.config;
      systemd.services.xremap = {
        description = "xremap";
        path = [ cfg.package ];
        wantedBy = [ "multi-user.target" ];
        serviceConfig = {
          PrivateNetwork = true;
          MemoryDenyWriteExecute = true;
          CapabilityBoundingSet = [ "~CAP_SETUID" "~CAP_SETGID" "~CAP_SETPCAP" "~CAP_SYS_ADMIN" "~CAP_SYS_PTRACE" "~CAP_NET_ADMIN" "~CAP_FOWNER" "~CAP_IPC_OWNER" "~CAP_SYS_TIME" "~CAP_KILL" "~CAP_SYS_BOOT" "~CAP_LINUX_IMMUTABLE" "~CAP_IPC_LOCK" "~CAP_SYS_CHROOT" "~CAP_BLOCK_SUSPEND" "~CAP_SYS_PACCT" "~CAP_WAKE_ALARM" "~CAP_AUDIT_WRITE" "~CAP_AUDIT_CONTROL" "~CAP_AUDIT_READ" "CAP_DAC_READ_SEARCH" "CAP_DAC_OVERRIDE" ];
          SystemCallArchitectures = [ "native" ];
          RestrictRealtime = true;
          SystemCallFilter = map (x: "~@${x}") [ "clock" "debug" "module" "reboot" "swap" "cpu-emulation" "obsolete" "privileged" "resources" ];
          LockPersonality = true;
          UMask = "077";
          IPAddressDeny = [ "0.0.0.0/0" "::/0" ];
          # ProtectClock adds to DeviceAllow, which does not seem to work with xremap since it tries to enumerate all /dev/input devices
          # ProtectClock = true;
          # DeviceAllow = [ "/dev/input/event24" ];
          ProtectHostname = true;
          # Does not work, the service cannot locate sway socket
          # PrivateUsers = true;
          RestrictAddressFamilies = "AF_UNIX";
          RestrictNamespaces = true;
          # RestrictNamespaces = ["~CLONE_NEWUSER" "~CLONE_NEWIPC" "~CLONE_NEWNET" "~CLONE_NEWNS" "~CLONE_NEWPID"];
          # ProtectClock = true;
          # Need 'tmpfs' here so that the socket may be actually bind-mounted through Bind*Paths
          ProtectHome = "tmpfs";
          # This is needed, otherwise xremap cannot read from sway socket
          BindReadOnlyPaths = [ userPath ];
          # Sway socket gets generated as $XDG_RUNTIME_DIR/sway-ipc.$UID.$SWAY_PID
          # Hacky way to allow sway socket
          # Systemd does not support wildcards :(
          InaccessiblePaths = map (x: "-${userPath}/${x}") [ "app" "bus" "dbus-1" ".dbus-proxy" "dconf" "env-vars" ".flatpak" ".flatpak-helper" "gnupg" "pipewire-0" "pipewire-0.lock" "pulse" "systemd" "tmux-${toString cfg.userId}" "wayland-1" "wayland-1.lock" ];
          PrivateTmp = true;
          ProtectKernelLogs = true;
          # Does not work, running as root
          # ProtectProc = true;
          # SystemCallFilter = "~@clock";
          NoNewPrivileges = true;
          ProtectSystem = "strict";
          ProtectKernelTunables = true;
          ProtectKernelModules = true;
          ProtectControlGroups = true;
          RestrictSUIDSGID = true;
          # End of hardening
          ExecStart = ''
            ${cfg.package}/bin/xremap --device "${cfg.deviceName}" ${if cfg.watch then "--watch" else ""} /etc/xremap_config.yml
          '';
          Nice = -20;
        };
      };
    };
}
