# TUI / 自动化工具专用入口。

{ lib, ... }:

{
  imports = [
    ./packages.nix
  ]
  ++ lib.optional (builtins.pathExists ./settings/default.nix) ./settings/default.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
}
