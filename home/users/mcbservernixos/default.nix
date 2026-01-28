# 用户入口（mcbservernixos）：选择 profile + 用户级文件。

{ ... }:

let
  user = "mcbservernixos";
in
{
  # 服务器用户使用 minimal profile
  imports = [
    ../../profiles/minimal.nix
    ./git.nix
  ];

  # Home Manager 基本信息
  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;

  xdg.enable = true;
}
