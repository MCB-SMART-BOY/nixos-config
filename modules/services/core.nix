{ config, pkgs, lib, ... }:

let
  netCaps = [
    "CAP_NET_ADMIN"
    "CAP_NET_BIND_SERVICE"
    "CAP_NET_RAW"
  ];
  proxyUrl = config.mcb.proxyUrl;
  proxyMode = config.mcb.proxyMode;
  proxyServiceEnabled = proxyMode == "tun";
  perUserTunEnabled = config.mcb.perUserTun.enable;
  userList =
    if config.mcb.users != [ ] then
      config.mcb.users
    else
      [ config.mcb.user ];
  proxyEnabled = proxyMode == "http" && proxyUrl != "";
  clashPath = lib.makeBinPath [
    pkgs.clash-verge-rev
    pkgs.mihomo
    pkgs.iproute2
    pkgs.iptables
    pkgs.coreutils
    pkgs.procps
  ];

  mkClashService = user:
    let
      clashHome = "/home/${user}";
      clashConfig = "${clashHome}/.config";
      clashData = "${clashHome}/.local/share";
      clashCache = "${clashHome}/.cache";
      clashState = "${clashHome}/.local/state";
    in
    {
      description = "Clash Verge Service Mode Daemon (${user})";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "simple";
        User = user;
        WorkingDirectory = clashHome;
        UMask = "0002";
      Environment = [
        "HOME=${clashHome}"
        "XDG_CONFIG_HOME=${clashConfig}"
        "XDG_DATA_HOME=${clashData}"
        "XDG_CACHE_HOME=${clashCache}"
        "XDG_STATE_HOME=${clashState}"
        "XDG_RUNTIME_DIR=/run/clash-verge-rev-${user}"
        "TMPDIR=/run/clash-verge-rev-${user}"
        "PATH=${clashPath}:/run/wrappers/bin"
      ];
        Restart = "on-failure";
        RestartSec = "2s";
        ExecStart = "${pkgs.clash-verge-rev}/bin/clash-verge-service";
        RuntimeDirectory = "clash-verge-rev-${user}";
      RuntimeDirectoryMode = "0700";
        CapabilityBoundingSet = netCaps;
        AmbientCapabilities = netCaps;
      };
    };
in
{
  services.openssh.enable = true;

  programs.nix-ld.enable = true;

  # Clash Verge service uses runtime IPC; isolate per-user runtime dirs to avoid conflicts.
  systemd.services = lib.mkMerge [
    (lib.mkIf (proxyServiceEnabled && !perUserTunEnabled) {
      clash-verge-service = mkClashService config.mcb.user;
    })
    (lib.mkIf (proxyServiceEnabled && perUserTunEnabled) (
      lib.listToAttrs (map (user: {
        name = "clash-verge-service@${user}";
        value = mkClashService user;
      }) userList)
    ))
    (lib.mkIf proxyEnabled {
      nix-daemon.environment = {
        https_proxy = proxyUrl;
        http_proxy = proxyUrl;
      };
    })
  ];

  services.mihomo = {
    enable = false;
    configFile = "/etc/mihomo/config.yaml";
  };

  systemd.services.mihomo = lib.mkIf config.services.mihomo.enable {
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];
    serviceConfig = {
      CapabilityBoundingSet = netCaps;
      AmbientCapabilities = netCaps;
      WorkingDirectory = "/var/lib/mihomo";
    };
  };
}
