# 用户完整 profile：桌面与开发常用功能集合。

{ ... }:

{
  # 桌面用户常用的完整功能集合
  imports = [
    ../modules/base.nix
    ../modules/programs.nix
    ../modules/desktop.nix
    ../modules/shell.nix
    ../modules/git.nix
  ];
}
