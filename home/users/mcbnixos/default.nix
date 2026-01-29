# 用户入口（mcbnixos）：选择 profile + 用户级文件。

{ lib, pkgs, ... }:

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

  # 使用自定义 .zshrc，避免与 Home Manager 自动生成冲突
  programs.zsh.enable = lib.mkForce false;
  home.packages = with pkgs; [
    oh-my-zsh
    zsh-autosuggestions
    zsh-syntax-highlighting
    zsh-completions
  ];

  # 启用 Home Manager 管理自身
  programs.home-manager.enable = true;

  # 启用 XDG 规范目录结构
  xdg.enable = true;
}
