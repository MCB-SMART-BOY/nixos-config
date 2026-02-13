# 主机配置（laptop）：按需覆盖 profile 与主机参数。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  hardwareFile =
    if builtins.pathExists ./hardware-configuration.nix then
      ./hardware-configuration.nix
    else if builtins.pathExists ../../hardware-configuration.nix then
      ../../hardware-configuration.nix
    else
      null;
  allUsers = if config.mcb.users != [ ] then config.mcb.users else [ config.mcb.user ];
  adminUsers = if config.mcb.adminUsers != [ ] then config.mcb.adminUsers else [ config.mcb.user ];
in
{
  imports = [
    ../profiles/desktop.nix
  ]
  ++ lib.optional (hardwareFile != null) hardwareFile
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  # 允许在仓库/CI 环境中评估 flake（此时通常没有机器私有 hardware-configuration.nix）
  fileSystems = lib.mkIf (hardwareFile == null) {
    "/" = {
      # 评估占位值：若用于真实部署会快速失败，避免误挂载错误磁盘。
      device = "/dev/disk/by-label/__MISSING_HARDWARE_CONFIGURATION__";
      fsType = "ext4";
    };
  };
  warnings = lib.optional (hardwareFile == null) ''
    hosts/laptop/hardware-configuration.nix 缺失；当前根文件系统为评估占位值，不可用于实际部署。
  '';

  mcb = {
    # 笔记本用户与代理设置
    user = "mcblaptopnixos";
    users = [ "mcblaptopnixos" ];
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
    perUserTun = {
      enable = true;
      redirectDns = true;
      interfaces = {
        mcblaptopnixos = "Meta";
      };
      dnsPorts = {
        mcblaptopnixos = 1053;
      };
    };

    hardware.gpu = {
      # 与 nixos 主机相同的 PCI busId（含 hybrid 特化）
      igpuVendor = "intel";
      prime = {
        intelBusId = "PCI:0:2:0";
        nvidiaBusId = "PCI:1:0:0";
      };
      nvidia.open = true;
      specialisations.enable = true;
      specialisations.modes = [
        "igpu"
        "hybrid"
        "dgpu"
      ];
    };
  };

  networking.hostName = "laptop";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  # 为每个用户创建私有组，避免共享 users 组导致跨用户目录权限扩大。
  users.groups = lib.genAttrs allUsers (_: { });

  # 创建系统用户
  users.users = lib.genAttrs allUsers (name: {
    isNormalUser = true;
    description = name;
    group = name;
    extraGroups =
      (lib.optionals (lib.elem name adminUsers) [ "wheel" ])
      ++ [
        "users"
        "networkmanager"
        "video"
        "audio"
      ]
      ++ lib.optionals config.virtualisation.docker.enable [ "docker" ]
      ++ lib.optionals config.virtualisation.libvirtd.enable [ "libvirtd" ];
    shell = pkgs.zsh;
    linger = true;
  });

}
