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
in
{
  imports = [
    ../profiles/desktop.nix
  ]
  ++ lib.optional (hardwareFile != null) hardwareFile
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

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
      specialisations.modes = [
        "igpu"
        "hybrid"
        "dgpu"
      ];
    };
  };

  boot.kernelPackages = pkgs.linuxPackages_latest;

  networking.hostName = "laptop";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  # 创建系统用户
  users.users = lib.genAttrs allUsers (name: {
    isNormalUser = true;
    description = name;
    extraGroups = [
      "wheel"
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
