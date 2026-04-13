# mcbctl-managed: home-managed-default
# mcbctl-checksum: ed0031500cf11e8a8c0065337af1767c7c7d3dd61c23a04db28f1fa7106bb205
# TUI / 自动化工具专用入口。

{ lib, ... }:

{
  imports = [
    ./packages.nix
  ]
  ++ lib.optional (builtins.pathExists ./settings/default.nix) ./settings/default.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
}
