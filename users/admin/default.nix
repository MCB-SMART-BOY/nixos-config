# 用户入口（admin）：管理员，预置的唯一用户。

{ lib, ... }:

let
  user = "admin";
in
{
  imports = [
    ./base.nix
    ./programs.nix
    ./desktop.nix
    ./shell.nix
    ./git.nix
    ./packages.nix
    ./noctalia.nix
    ./files.nix
    ./scripts.nix
  ]
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  # Home Manager 基本信息
  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  # 启用 Home Manager 管理自身
  programs.home-manager.enable = true;

  # 启用 XDG 规范目录结构
  xdg.enable = true;

}
