# 机器管理的用户设置聚合入口。

{ lib, ... }:

let
  splitImports = lib.concatLists [
    (lib.optional (builtins.pathExists ./desktop.nix) ./desktop.nix)
    (lib.optional (builtins.pathExists ./session.nix) ./session.nix)
    (lib.optional (builtins.pathExists ./mime.nix) ./mime.nix)
  ];
  legacySettings =
    lib.optional ((splitImports == [ ]) && builtins.pathExists ../settings.nix) ../settings.nix;
in
{
  imports = splitImports ++ legacySettings;
}
