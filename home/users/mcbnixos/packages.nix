# 用户个人软件入口（mcbnixos）。
# 在这里声明“仅该用户可见”的 Home Manager 包，不影响其他用户可见性。

{ lib, ... }:

{
  imports = [ ../../modules/personal-packages.nix ];

  mcb.personalPackages = {
    enableZed = lib.mkDefault true;
    zedChannel = lib.mkDefault "official-stable";
    enableYesPlayMusic = lib.mkDefault true;
  };
}
