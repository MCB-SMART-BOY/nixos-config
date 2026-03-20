# 用户个人软件入口（mcbservernixos）。

{ lib, ... }:

{
  imports = [ ../../modules/personal-packages.nix ];

  mcb.personalPackages = {
    enableZed = lib.mkDefault false;
    zedChannel = lib.mkDefault "official-stable";
    enableYesPlayMusic = lib.mkDefault false;
  };
}
