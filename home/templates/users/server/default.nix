# 用户模板示例（server）：这里保留的是服务器用户入口样板，不会被 Home Manager 自动加载。

{ lib, ... }:

let
  user = "your-user";
in
{
  # 服务器用户使用 minimal profile
  imports = [
    ../../profiles/minimal.nix
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
}
