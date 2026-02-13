# 主机配置（server）：按需覆盖 profile 与主机参数。

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
    ../profiles/server.nix
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
    hosts/server/hardware-configuration.nix 缺失；当前根文件系统为评估占位值，不可用于实际部署。
  '';

  mcb = {
    # 服务器用户与最小化代理设置
    user = "mcbservernixos";
    users = [ "mcbservernixos" ];
    cpuVendor = "intel";
    proxyMode = "off";

    hardware.gpu = {
      # 服务器默认不提供 GPU 特化入口
      specialisations.enable = false;
    };
  };

  networking.hostName = "server";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  # 为每个用户创建私有组，避免共享 users 组导致跨用户目录权限扩大。
  users.groups = lib.genAttrs allUsers (_: { });

  # 创建系统用户（服务器角色较精简）
  users.users = lib.genAttrs allUsers (name: {
    isNormalUser = true;
    description = name;
    group = name;
    extraGroups =
      (lib.optionals (lib.elem name adminUsers) [ "wheel" ])
      ++ [
        "users"
        "networkmanager"
      ]
      ++ lib.optionals config.virtualisation.docker.enable [ "docker" ]
      ++ lib.optionals config.virtualisation.libvirtd.enable [ "libvirtd" ];
    shell = pkgs.zsh;
    linger = true;
  });
}
