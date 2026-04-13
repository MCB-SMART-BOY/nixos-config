# mcbctl-managed: home-settings-default
# mcbctl-checksum: d7936c5babe8df4fe312116251b98565d147e7982af54a222d289c87d05e99e1
# 机器管理的用户设置聚合入口。

{ lib, ... }:

let
  splitImports = lib.concatLists [
    (lib.optional (builtins.pathExists ./desktop.nix) ./desktop.nix)
    (lib.optional (builtins.pathExists ./session.nix) ./session.nix)
    (lib.optional (builtins.pathExists ./mime.nix) ./mime.nix)
  ];
in
{
  imports = splitImports;
}
