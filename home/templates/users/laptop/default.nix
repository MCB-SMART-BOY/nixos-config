# 用户模板示例（laptop）：这里保留的是桌面用户入口样板，不会被 Home Manager 自动加载。

{ lib, ... }:

let
  user = "your-user";
in
{
  # 笔记本用户使用完整桌面 profile
  imports = [
    ../../profiles/full.nix
    ./git.nix
    ./packages.nix
  ]
  ++ lib.optional (builtins.pathExists ./managed/default.nix) ./managed/default.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  # Home Manager 基本信息
  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;

  xdg.enable = true;

  # 默认使用标准 Noctalia 顶栏（无自定义脚本依赖）
  mcb.noctalia.barProfile = "default";

  # Flatpak 版本由系统级 Flatpak 提供桌面入口
}
