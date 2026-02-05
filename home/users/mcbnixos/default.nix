# 用户入口（mcbnixos）：选择 profile + 用户级文件。

{ ... }:

let
  user = "mcbnixos";
in
{
  # 该用户启用完整桌面 profile
  imports = [
    ../../profiles/full.nix
    ./git.nix
    ./files.nix
    ./scripts.nix
  ];

  # Home Manager 基本信息
  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  # 启用 Home Manager 管理自身
  programs.home-manager.enable = true;

  # 启用 XDG 规范目录结构
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
