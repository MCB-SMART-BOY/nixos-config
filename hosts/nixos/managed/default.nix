# TUI / 自动化工具专用主机入口。

{ lib, ... }:

let
  splitImports = lib.concatLists [
    (lib.optional (builtins.pathExists ./users.nix) ./users.nix)
    (lib.optional (builtins.pathExists ./network.nix) ./network.nix)
    (lib.optional (builtins.pathExists ./gpu.nix) ./gpu.nix)
    (lib.optional (builtins.pathExists ./virtualization.nix) ./virtualization.nix)
  ];
in
{
  imports = splitImports ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
}
