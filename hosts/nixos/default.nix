# 主机配置（nixos）：指定用户、代理模式与主机级参数。
# 新手提示：这里是“主机层”的总入口，会导入 profiles + 硬件配置。

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
    hosts/nixos/hardware-configuration.nix 缺失；当前根文件系统为评估占位值，不可用于实际部署。
  '';

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
      # 默认关闭 per-user DNS 重定向，避免未配置监听端口时断网
      redirectDns = false;
      interfaces = {
        mcbnixos = "Meta";
        mcblaptopnixos = "Mihomo";
      };
      dnsPorts = {
        mcbnixos = 1053;
        mcblaptopnixos = 1054;
      };
    };

    hardware.gpu = {
      # Hybrid 特化需要 busId（iGPU + NVIDIA）
      igpuVendor = "intel";
      prime = {
        intelBusId = "PCI:0:2:0";
        nvidiaBusId = "PCI:1:0:0";
      };
      nvidia.open = true;
      specialisations.enable = true;
      # 覆盖特化模式列表，加入 hybrid
      specialisations.modes = [
        "igpu"
        "hybrid"
        "dgpu"
      ];
    };
  };

  networking.hostName = "nixos";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  # 为每个用户创建私有组，避免共享 users 组导致跨用户目录权限扩大。
  users.groups = lib.genAttrs allUsers (_: { });

  # 创建系统用户并按需加入组
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
