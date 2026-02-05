# 用户入口（mcblaptopnixos）：选择 profile + 用户级文件。

{ ... }:

let
  user = "mcblaptopnixos";
in
{
  # 笔记本用户使用完整桌面 profile
  imports = [
    ../../profiles/full.nix
    ./git.nix
  ];

  # Home Manager 基本信息
  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;

  xdg.enable = true;

  # 覆盖桌面入口：让 wemeet 启动 XWayland 版本
  xdg.desktopEntries.wemeet = {
    name = "Tencent Meeting";
    genericName = "Video Conference";
    comment = "Tencent Meeting (XWayland)";
    exec = "wemeet-xwayland %U";
    icon = "wemeet";
    terminal = false;
    categories = [
      "Network"
      "VideoConference"
      "AudioVideo"
    ];
  };
}
