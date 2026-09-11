{ ... }:

{
  config = {
    programs.steam = {
      enable = true;
      remotePlay.openFirewall = true;
      gamescopeSession.enable = false;
    };

    programs.gamemode.enable = true;
  };
}
