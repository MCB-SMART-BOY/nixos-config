# 主机配置（nixos）：指定用户、代理模式与主机级参数。
# 新手提示：这里是“主机层”的总入口，会导入 profiles + 硬件配置。

{ config, lib, pkgs, ... }:

let
  hardwareFile =
    if builtins.pathExists ./hardware-configuration.nix then
      ./hardware-configuration.nix
    else if builtins.pathExists ../../hardware-configuration.nix then
      ../../hardware-configuration.nix
    else
      null;
  allUsers =
    if config.mcb.users != [ ] then
      config.mcb.users
    else
      [ config.mcb.user ];
in
{
  imports =
    [ ../profiles/desktop.nix ]
    ++ lib.optional (hardwareFile != null) hardwareFile
    ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  mcb = {
    # 主用户与用户列表（影响 Home Manager 与权限）
    user = "mcbnixos";
    users = [
      "mcbnixos"
      "mcblaptopnixos"
    ];
    # 代理与 TUN 相关参数
    tunInterface = "Meta";
    tunInterfaces = [
      "Meta"
      "Mihomo"
      "clash0"
    ];
    cpuVendor = "intel";
    proxyMode = "tun";
    proxyUrl = "";
    enableProxyDns = false;
    proxyDnsAddr = "127.0.0.1";
    proxyDnsPort = 53;
    # 每个用户独立 TUN（高级用法）
    perUserTun = {
      enable = true;
      redirectDns = true;
      interfaces = {
        mcbnixos = "Meta";
        mcblaptopnixos = "Mihomo";
      };
      dnsPorts = {
        mcbnixos = 1053;
        mcblaptopnixos = 1054;
      };
    };
  };

  networking.hostName = "nixos";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  # 创建系统用户并加入常用组
  users.users = lib.genAttrs allUsers (name: {
    isNormalUser = true;
    description = name;
    extraGroups = [
      "wheel"
      "networkmanager"
      "video"
      "audio"
      "docker"
      "libvirtd"
    ];
    shell = pkgs.zsh;
    linger = true;
  });

  systemd.tmpfiles.rules =
    (lib.concatLists (map (name: [
      "d /home/${name}/.config/clash-verge 2775 ${name} users -"
      "d /home/${name}/.config/clash-verge-rev 2775 ${name} users -"
    ]) allUsers))
    ++ [
      "d /var/lib/mihomo 0755 root root -"
    ];
}
