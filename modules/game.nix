{ ... }:

{
  programs = {
    steam = {
      enable = true;
      remotePlay.openFirewall = true;
      gamescopeSession.enable = false;
    };
    gamemode.enable = true;
  };
}
