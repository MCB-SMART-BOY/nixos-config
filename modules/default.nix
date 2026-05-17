# 系统模块聚合入口：导入所有子模块 + 设定通用默认值。
# 克隆后直接可用；机器特定值请通过项目根目录的 local.nix 覆盖（mkForce）。

{ lib, ... }:

{
  imports = [
    # 核心
    ./options.nix
    ./users.nix
    ./boot.nix
    ./networking.nix
    ./security.nix
    ./nix.nix
    ./packages.nix
    ./services/core.nix
    # 桌面扩展
    ./i18n.nix
    ./fonts.nix
    ./desktop.nix
    ./services/desktop.nix
    ./virtualization.nix
    ./gaming.nix
  ];

  # ── 以下全部使用 mkDefault，根目录 local.nix 可用 mkForce 覆盖 ──

  mcb = {
    hostRole = lib.mkDefault "desktop";
    userLinger = lib.mkDefault true;
    user = lib.mkDefault "admin";
    users = lib.mkDefault [ "admin" ];
    adminUsers = lib.mkDefault [ "admin" ];

    cpuVendor = lib.mkDefault "intel";
    proxyMode = lib.mkDefault "tun";
    proxyUrl = lib.mkDefault "";


    packages = {
      enableNetworkCli = lib.mkDefault true;
      enableNetworkGui = lib.mkDefault true;
      enableShellTools = lib.mkDefault true;
      enableWaylandTools = lib.mkDefault true;
      enableSystemTools = lib.mkDefault true;
      enableTheming = lib.mkDefault true;
      enableXorgCompat = lib.mkDefault true;

      enableGeekTools = lib.mkDefault false;
      enableMusic = lib.mkDefault false;
    };

    flatpak.enable = lib.mkDefault false;
  };

  networking.hostName = lib.mkDefault "nixos";
  system.stateVersion = lib.mkDefault "25.11";

  hardware.graphics.enable = true;
  programs.zsh.enable = true;
}
