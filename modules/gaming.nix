# 游戏支持：Steam、gamemode、mangohud。
# 使用 NixOS 原生选项；在 local.nix 中以 mkForce 覆盖即可关闭。
#   programs.steam.enable = lib.mkForce false;
#   programs.gamemode.enable = lib.mkForce false;

{
  config,
  lib,
  pkgs,
  ...
}:

{
  config = {
    programs.steam = {
      enable = lib.mkDefault true;
      remotePlay.openFirewall = true;
      gamescopeSession.enable = false;
      extraCompatPackages = with pkgs; [
        mangohud
        gamemode
      ];
    };

    programs.gamemode.enable = lib.mkDefault true;
  };
}
