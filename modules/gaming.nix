{ pkgs, ... }:

{
  programs.steam = {
    enable = true;
    remotePlay.openFirewall = true;
    gamescopeSession.enable = true;
    extraCompatPackages = with pkgs; [
      mangohud
      gamemode
    ];
  };

  programs.gamemode.enable = true;
}
