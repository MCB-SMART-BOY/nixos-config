{ pkgs, vars, lib, ... }:

let
  netCaps = [
    "CAP_NET_ADMIN"
    "CAP_NET_BIND_SERVICE"
    "CAP_NET_RAW"
  ];
  proxyUrl = vars.proxyUrl;
in
{
  services.openssh.enable = true;

  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
  };

  services.tlp.enable = true;

  programs.nix-ld.enable = true;

  programs.appimage = {
    enable = true;
    binfmt = true;
  };

  hardware.graphics = {
    enable = true;
    enable32Bit = true;
    extraPackages = with pkgs; [
      intel-media-driver
      libvdpau-va-gl
    ];
  };

  systemd.services.clash-verge-service = {
    description = "Clash Verge Service Mode Daemon";
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "simple";
      Restart = "always";
      ExecStart = "${pkgs.clash-verge-rev}/bin/clash-verge-service";
      CapabilityBoundingSet = netCaps;
      AmbientCapabilities = netCaps;
      DevicePolicy = "closed";
      DeviceAllow = [ "/dev/net/tun rwm" ];
      LockPersonality = true;
      MemoryDenyWriteExecute = true;
      NoNewPrivileges = true;
      PrivateTmp = true;
      ProtectClock = true;
      ProtectControlGroups = true;
      ProtectHostname = true;
      ProtectKernelLogs = true;
      ProtectKernelModules = true;
      ProtectKernelTunables = true;
      RestrictRealtime = true;
      RestrictSUIDSGID = true;
      SystemCallArchitectures = "native";
    };
  };

  systemd.services.nix-daemon.environment = lib.mkIf (proxyUrl != "") {
    https_proxy = proxyUrl;
    http_proxy = proxyUrl;
  };

  services.mihomo = {
    enable = false;
    configFile = "/etc/mihomo/config.yaml";
  };

  systemd.services.mihomo = {
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];
    serviceConfig = {
      User = "root";
      Group = "root";
      CapabilityBoundingSet = netCaps;
      AmbientCapabilities = netCaps;
      WorkingDirectory = "/var/lib/mihomo";
    };
  };
}
