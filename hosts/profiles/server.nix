# 服务器 profile：更精简的系统模块组合。

{ ... }:

{
  # 服务器 profile：基于 base，并关闭桌面相关包组
  imports = [
    ./base.nix
  ];

  mcb.packages = {
    enableNetwork = true;
    enableShellTools = true;
    enableWaylandTools = false;
    enableBrowsersAndMedia = false;
    enableDev = false;
    enableChat = false;
    enableEmulation = false;
    enableEntertainment = false;
    enableGaming = false;
    enableSystemTools = true;
    enableTheming = false;
    enableXorgCompat = false;
    enableGeekTools = false;
  };
}
