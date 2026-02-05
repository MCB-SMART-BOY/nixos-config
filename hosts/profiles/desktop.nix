# 桌面 profile：启用桌面模块与常用包组。

{ ... }:

{
  # 桌面主机引入完整的系统模块
  imports = [
    ./base.nix
    ../../modules/i18n.nix
    ../../modules/fonts.nix
    ../../modules/desktop.nix
    ../../modules/services/desktop.nix
    ../../modules/virtualization.nix
    ../../modules/gaming.nix
  ];

  mcb.packages = {
    # 桌面常用包组全部开启
    enableNetworkCli = true;
    enableNetworkGui = true;
    enableShellTools = true;
    enableWaylandTools = true;
    enableBrowsersAndMedia = true;
    enableDev = true;
    enableChat = true;
    enableEmulation = true;
    enableEntertainment = true;
    enableGaming = true;
    enableSystemTools = true;
    enableTheming = true;
    enableXorgCompat = true;
    enableGeekTools = true;
    enableOffice = true;
    enableLife = true;
    enableAnime = true;
    enableMusic = true;
  };
}
