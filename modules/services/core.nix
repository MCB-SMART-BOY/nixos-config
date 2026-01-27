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
      runtimeDirName = "clash-verge-rev";
      userGroup = lib.attrByPath [ user "group" ] "users" config.users.users;
    in
    {
      description = "Clash Verge Service Mode Daemon (${user})";
      after = [ "network-online.target" ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "simple";
        User = user;
        Group = userGroup;
        WorkingDirectory = clashHome;
        UMask = "0002";
        PermissionsStartOnly = true;
        RuntimeDirectory = runtimeDirName;
        RuntimeDirectoryMode = "0700";
        ExecStartPre = [
          (pkgs.writeShellScript "clash-verge-prestart-${user}" ''
            set -euo pipefail
            uid="$(${pkgs.coreutils}/bin/id -u ${user})"
            runtime_dir="/run/${runtimeDirName}"
            for dir in \
              "${clashConfig}/clash-verge" \
              "${clashConfig}/clash-verge-rev" \
              "${clashData}/clash-verge" \
              "${clashData}/clash-verge-rev" \
              "${clashCache}/clash-verge-rev" \
              "${clashState}/clash-verge-rev"; do
              ${pkgs.coreutils}/bin/install -d -m 2775 -o ${user} -g ${userGroup} "$dir"
            done
            ${pkgs.coreutils}/bin/chown -R ${user}:${userGroup} \
              "${clashConfig}/clash-verge" \
              "${clashConfig}/clash-verge-rev" \
              "${clashData}/clash-verge" \
              "${clashData}/clash-verge-rev" \
              "${clashCache}/clash-verge-rev" \
              "${clashState}/clash-verge-rev" \
              2>/dev/null || true
            rm -f "$runtime_dir"/*.sock 2>/dev/null || true
          '')
        ];
        Environment = [
          "HOME=${clashHome}"
          "XDG_CONFIG_HOME=${clashConfig}"
          "XDG_DATA_HOME=${clashData}"
          "XDG_CACHE_HOME=${clashCache}"
          "XDG_STATE_HOME=${clashState}"
          "XDG_RUNTIME_DIR=/run/${runtimeDirName}"
          "TMPDIR=/run/${runtimeDirName}"
          "PATH=${clashPath}:/run/wrappers/bin"
        ];
        Restart = "on-failure";
        RestartSec = "2s";
        ExecStart = "${pkgs.clash-verge-rev}/bin/clash-verge-service";
        DeviceAllow = [ "/dev/net/tun rw" ];
        CapabilityBoundingSet = netCaps;
        AmbientCapabilities = netCaps;
      };
    };
in
{
  services.openssh.enable = true;

  programs.nix-ld.enable = true;

  systemd.tmpfiles.rules = lib.optionals proxyServiceEnabled (
    lib.concatLists (map (user:
      let
        userGroup = lib.attrByPath [ user "group" ] "users" config.users.users;
      in
      [
        "d /home/${user}/.config/clash-verge 2775 ${user} ${userGroup} -"
        "d /home/${user}/.config/clash-verge-rev 2775 ${user} ${userGroup} -"
        "d /home/${user}/.local/share/clash-verge 2775 ${user} ${userGroup} -"
        "d /home/${user}/.local/share/clash-verge-rev 2775 ${user} ${userGroup} -"
        "d /home/${user}/.cache/clash-verge-rev 2775 ${user} ${userGroup} -"
        "d /home/${user}/.local/state/clash-verge-rev 2775 ${user} ${userGroup} -"
      ]) userList)
  );

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
    (lib.mkIf config.services.mihomo.enable {
      mihomo = {
        after = [ "network-online.target" ];
        wants = [ "network-online.target" ];
        serviceConfig = {
          CapabilityBoundingSet = netCaps;
          AmbientCapabilities = netCaps;
          WorkingDirectory = "/var/lib/mihomo";
        };
      };
    })
  ];

  services.mihomo = {
    enable = false;
    configFile = "/etc/mihomo/config.yaml";
  };

  # systemd.services.mihomo merged above to avoid duplicate attribute definitions.
}
