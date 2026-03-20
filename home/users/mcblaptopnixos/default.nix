# 用户入口（mcblaptopnixos）：选择 profile + 用户级文件。

{ lib, ... }:

let
  user = "mcblaptopnixos";
in
{
  # 笔记本用户使用完整桌面 profile
  imports = [
    ../../profiles/full.nix
    ./git.nix
    ./packages.nix
  ] ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

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
