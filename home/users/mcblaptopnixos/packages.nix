# 用户个人软件入口（mcblaptopnixos）。

{ lib, ... }:

{
  imports = [ ../../modules/personal-packages.nix ];

  mcb.personalPackages = {
    enableZed = lib.mkDefault true;
    zedChannel = lib.mkDefault "official-stable";
    enableYesPlayMusic = lib.mkDefault true;
  };
}
