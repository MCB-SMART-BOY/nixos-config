# 网络基础配置：NetworkManager、防火墙、DNS、HTTP 代理。
# TUN 模式由 clash-verge-rev GUI 自行管理，不在此处做策略路由。

{ config, lib, ... }:

let
  proxyMode = config.mcb.proxyMode;
  proxyUrl = config.mcb.proxyUrl;
  proxyEnabled = proxyMode == "http" && proxyUrl != "";
in
{
  assertions = [
    {
      assertion = proxyMode != "http" || proxyUrl != "";
      message = "mcb.proxyMode = \"http\" requires a non-empty mcb.proxyUrl.";
    }
  ];

  networking = {
    networkmanager = {
      enable = true;
      dns = "systemd-resolved";
    };

    # HTTP 代理（仅 proxyMode = http 时启用）
    proxy = lib.mkIf proxyEnabled {
      default = proxyUrl;
      noProxy = "127.0.0.1,localhost,internal.domain";
    };

    firewall = {
      enable = true;
      checkReversePath = "strict";
      allowedTCPPorts = [ 22 ];
      trustedInterfaces =
        lib.optionals config.virtualisation.docker.enable [ "docker0" ]
        ++ lib.optionals config.virtualisation.libvirtd.enable [ "virbr0" ];
    };
  };

  services.resolved = {
    enable = true;
    fallbackDns = [
      "223.5.5.5"
      "1.1.1.1"
    ];
  };
}
