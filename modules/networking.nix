# 网络与代理核心逻辑：DNS、proxy、TUN 路由、策略路由。
# 是本项目最复杂的模块之一，改动前务必理解流程。
# 新手提示：大部分输入来自 hosts/*/default.nix 的 mcb.* 选项。

{
  config,
  lib,
  options,
  pkgs,
  ...
}:

let
  # === 读取 mcb.* 基础选项 ===
  scriptsRs = pkgs.callPackage ../pkgs/scripts-rs { };
  tunInterface = config.mcb.tunInterface;
  tunInterfaces = config.mcb.tunInterfaces;
  perUserTunEnabled = config.mcb.perUserTun.enable;
  perUserInterfaces = config.mcb.perUserTun.interfaces;
  perUserDnsRedirect = perUserTunEnabled && config.mcb.perUserTun.redirectDns;
  perUserDnsPorts = config.mcb.perUserTun.dnsPorts;
  userList = if config.mcb.users != [ ] then config.mcb.users else [ config.mcb.user ];
  # 合并所有可被信任的 TUN 接口（用于防火墙与策略路由）
  tunInterfacesEffective = lib.unique (
    (lib.optionals (tunInterface != "") [ tunInterface ])
    ++ tunInterfaces
    ++ (lib.optionals perUserTunEnabled (lib.attrValues perUserInterfaces))
  );
  proxyUrl = config.mcb.proxyUrl;
  proxyMode = config.mcb.proxyMode;
  proxyServiceEnabled = proxyMode == "tun";
  proxyEnabled = proxyMode == "http" && proxyUrl != "";
  proxyDnsEnabled = proxyServiceEnabled && config.mcb.enableProxyDns;
  proxyDnsAddr = config.mcb.proxyDnsAddr;
  proxyDnsPort = config.mcb.proxyDnsPort;
  proxyDnsTarget =
    if proxyDnsPort == 53 then proxyDnsAddr else "${proxyDnsAddr}:${toString proxyDnsPort}";
  resolvedHasDns = lib.hasAttrByPath [ "services" "resolved" "dns" ] options;
  resolvedHasFallback = lib.hasAttrByPath [ "services" "resolved" "fallbackDns" ] options;
  # systemd-resolved 额外配置（避免与已有选项重复）
  resolvedExtraConfig = ''
    ${lib.optionalString (!resolvedHasDns && proxyDnsEnabled) "DNS=${proxyDnsTarget}"}
    ${lib.optionalString (!resolvedHasFallback && !proxyDnsEnabled) "FallbackDNS=223.5.5.5 1.1.1.1"}
  '';

  # 为“每个用户单独 TUN”生成 systemd oneshot 服务
  mkRouteService =
    idx: user:
    let
      iface = perUserInterfaces.${user};
      tableId = config.mcb.perUserTun.tableBase + idx;
      priority = config.mcb.perUserTun.priorityBase + idx;
      dnsPort = perUserDnsPorts.${user} or 0;
      dnsPortStr = toString dnsPort;
      routeCommand =
        "${scriptsRs}/bin/mcb-tun-route-rs start --user ${user} --iface ${iface} --table-id ${toString tableId} --priority ${toString priority} --dns-port ${dnsPortStr}"
        + lib.optionalString perUserDnsRedirect " --redirect-dns";
      stopCommand =
        "${scriptsRs}/bin/mcb-tun-route-rs stop --user ${user} --iface ${iface} --table-id ${toString tableId} --priority ${toString priority} --dns-port ${dnsPortStr}"
        + lib.optionalString perUserDnsRedirect " --redirect-dns";
    in
    {
      description = "Per-user TUN routing (${user})";
      after = [
        "network-online.target"
        "clash-verge-service@${user}.service"
      ];
      partOf = [ "clash-verge-service@${user}.service" ];
      bindsTo = [ "clash-verge-service@${user}.service" ];
      wants = [
        "network-online.target"
        "clash-verge-service@${user}.service"
      ];
      wantedBy = [ "clash-verge-service@${user}.service" ];
      path = [
        pkgs.coreutils
        pkgs.iproute2
        pkgs.iptables
      ];
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        ExecStart = routeCommand;
        ExecStop = stopCommand;
        Restart = "on-failure";
        RestartSec = "2s";
      };
    };
in
{
  # === 参数校验：开启 per-user TUN 时必须满足的前置条件 ===
  assertions = [
    {
      assertion = proxyMode != "http" || proxyUrl != "";
      message = "mcb.proxyMode = \"http\" requires a non-empty mcb.proxyUrl.";
    }
    {
      assertion = !proxyServiceEnabled || perUserTunEnabled || tunInterface != "";
      message = "mcb.proxyMode = \"tun\" requires mcb.tunInterface when per-user TUN is disabled.";
    }
    {
      assertion = !proxyDnsEnabled || proxyDnsAddr != "";
      message = "mcb.enableProxyDns = true requires a non-empty mcb.proxyDnsAddr.";
    }
  ]
  ++ lib.optionals perUserTunEnabled [
    {
      assertion = proxyMode == "tun";
      message = "mcb.perUserTun.enable requires mcb.proxyMode = \"tun\".";
    }
    {
      assertion = config.mcb.enableProxyDns == false;
      message = "per-user TUN does not support global proxy DNS. Set mcb.enableProxyDns = false.";
    }
    {
      assertion = lib.all (user: lib.hasAttr user perUserInterfaces) userList;
      message = "mcb.perUserTun.interfaces must define a TUN interface for each user in mcb.users.";
    }
    {
      assertion =
        lib.length (lib.unique (lib.attrValues perUserInterfaces))
        == lib.length (lib.attrValues perUserInterfaces);
      message = "mcb.perUserTun.interfaces must use unique interface names per user.";
    }
  ]
  ++ lib.optionals perUserDnsRedirect [
    {
      assertion = lib.all (user: lib.hasAttr user perUserDnsPorts) userList;
      message = "mcb.perUserTun.dnsPorts must define a port for each user when redirectDns is enabled.";
    }
    {
      assertion =
        lib.length (lib.unique (lib.attrValues perUserDnsPorts))
        == lib.length (lib.attrValues perUserDnsPorts);
      message = "mcb.perUserTun.dnsPorts must use unique ports per user.";
    }
  ];

  networking = {
    networkmanager = {
      enable = true;
      dns = "systemd-resolved";
    };

    # HTTP 代理模式（仅 proxyMode = http 时启用）
    proxy = lib.mkIf proxyEnabled {
      default = proxyUrl;
      noProxy = "127.0.0.1,localhost,internal.domain";
    };

    # 防火墙：允许代理 DNS/本地面板端口，同时信任 TUN 接口
    firewall = {
      enable = true;
      # 默认使用严格反向路径检查，减少源地址伪造风险。
      checkReversePath = "strict";
      allowedTCPPorts = [
        22
      ];
      allowedUDPPorts = lib.optionals (proxyDnsEnabled && tunInterfacesEffective == [ ]) [ proxyDnsPort ];
      interfaces =
        (lib.optionalAttrs proxyDnsEnabled (
          lib.genAttrs tunInterfacesEffective (_: {
            allowedUDPPorts = [ proxyDnsPort ];
          })
        ))
        // (lib.optionalAttrs proxyServiceEnabled {
          lo = {
            allowedTCPPorts = [
              7890
              9090
            ];
          };
        });
      trustedInterfaces =
        tunInterfacesEffective
        ++ lib.optionals config.virtualisation.docker.enable [ "docker0" ]
        ++ lib.optionals config.virtualisation.libvirtd.enable [ "virbr0" ];
    };
  };

  systemd.services = lib.mkIf (proxyServiceEnabled && perUserTunEnabled) (
    lib.listToAttrs (
      lib.imap0 (idx: user: {
        name = "mcb-tun-route@${user}";
        value = mkRouteService idx user;
      }) userList
    )
  );

  systemd.paths = lib.mkIf (proxyServiceEnabled && perUserTunEnabled) (
    lib.listToAttrs (
      lib.imap0 (
        idx: user:
        let
          iface = perUserInterfaces.${user};
        in
        {
          name = "mcb-tun-route@${user}";
          value = {
            wantedBy = [ "multi-user.target" ];
            pathConfig = {
              PathExists = "/sys/class/net/${iface}";
              Unit = "mcb-tun-route@${user}.service";
            };
          };
        }
      ) userList
    )
  );

  services.resolved = {
    enable = true;
  }
  // lib.optionalAttrs (resolvedHasDns && proxyDnsEnabled) {
    dns = [ proxyDnsTarget ];
  }
  // lib.optionalAttrs (resolvedHasFallback && !proxyDnsEnabled) {
    fallbackDns = [
      "223.5.5.5"
      "1.1.1.1"
    ];
  }
  // lib.optionalAttrs (resolvedExtraConfig != "") {
    extraConfig = resolvedExtraConfig;
  };
}
