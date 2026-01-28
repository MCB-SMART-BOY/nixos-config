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
}
