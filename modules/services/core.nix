# 核心服务：ssh、代理服务（clash/mihomo）与运行时目录。
# 代理相关服务与网络模块紧密配合。
# 注意：proxyMode 在 hosts/*/default.nix 中设置。

{ config, pkgs, lib, ... }:

let
  # 代理服务需要的网络权限能力
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

  # 为指定用户生成 clash-verge-service 的 systemd 服务
  mkClashService = user:
    let
      clashHome = "/home/${user}";
      clashConfig = "${clashHome}/.config";
      clashData = "${clashHome}/.local/share";
      clashCache = "${clashHome}/.cache";
      clashState = "${clashHome}/.local/state";
      runtimeDirName = "clash-verge-rev-${user}";
      userGroup = lib.attrByPath [ user "group" ] "users" config.users.users;
      tunDevice =
        if perUserTunEnabled then
          (config.mcb.perUserTun.interfaces.${user} or "")
        else
          config.mcb.tunInterface;
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
        # clash-verge-service 固定使用 /run/clash-verge-rev/service.sock
        # 用 bind mount 将其映射到每个用户的运行时目录，避免多用户冲突
        BindPaths = [
          "/run/${runtimeDirName}:/run/clash-verge-rev"
        ];
        ExecStartPre = [
          (pkgs.writeShellScript "clash-verge-prestart-${user}" ''
            set -euo pipefail
            uid="$(${pkgs.coreutils}/bin/id -u ${user})"
            runtime_dir="/run/${runtimeDirName}"
            iface="${tunDevice}"
            ip="${pkgs.iproute2}/bin/ip"
            # 确保配置/数据目录存在并归属正确
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
            if [[ -n "$iface" ]]; then
              if ! "$ip" link show dev "$iface" >/dev/null 2>&1; then
                "$ip" tuntap add dev "$iface" mode tun user "$uid"
              fi
              "$ip" link set dev "$iface" up || true
            fi
          '')
        ];
        Environment = [
          # 统一 XDG 目录，避免应用乱写文件
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
  # 允许远程 SSH 连接
  services.openssh.enable = true;

  # 让非 Nix 动态链接程序可运行（需要时启用）
  programs.nix-ld.enable = true;

  # 为代理服务准备所需目录（仅 proxyMode=tun 时）
  systemd.tmpfiles.rules =
    lib.optionals proxyServiceEnabled (
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
      ++ [
        # GUI 仍使用固定 IPC 路径：/run/clash-verge-rev/service.sock
        # 这里把它指向主用户的 runtime 目录，保证 GUI 能连接
        "d /run/clash-verge-rev 0755 root root -"
        "L+ /run/clash-verge-rev/service.sock - - - - /run/clash-verge-rev-${config.mcb.user}/service.sock"
      ]
    )
    ++ lib.optionals config.services.mihomo.enable [
      "d /var/lib/mihomo 0755 root root -"
    ];

  # Clash Verge 使用 runtime IPC；多用户时隔离运行时目录避免冲突
  systemd.services = lib.mkMerge [
    # 单用户代理服务
    (lib.mkIf (proxyServiceEnabled && !perUserTunEnabled) {
      clash-verge-service = mkClashService config.mcb.user;
    })
    # 多用户代理服务（每个用户一个实例）
    (lib.mkIf (proxyServiceEnabled && perUserTunEnabled) (
      lib.listToAttrs (map (user: {
        name = "clash-verge-service@${user}";
        value = mkClashService user;
      }) userList)
    ))
    # http 代理模式时，把代理注入 nix-daemon 环境
    (lib.mkIf proxyEnabled {
      nix-daemon.environment = {
        https_proxy = proxyUrl;
        http_proxy = proxyUrl;
      };
    })
    # mihomo 需要额外能力与工作目录
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
    # 默认关闭，按需在 host 层开启
    enable = false;
    configFile = "/etc/mihomo/config.yaml";
  };

  # systemd.services.mihomo merged above to avoid duplicate attribute definitions.
}
