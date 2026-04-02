# 系统包组装配：把兼容警告与 environment.systemPackages 接到模块输出。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  packageDefs = import ./group-defs.nix {
    inherit config lib pkgs;
  };
in
{
  config = {
    warnings = lib.optionals (packageDefs.enabledLegacyUserScopedToggles != [ ]) [
      "mcb.packages.${lib.concatStringsSep ", mcb.packages." packageDefs.enabledLegacyUserScopedToggles} are compatibility toggles and do not install packages anymore. Declare user apps explicitly in home/users/<user>/packages.nix."
    ];

    environment.systemPackages = packageDefs.groups;
  };
}
