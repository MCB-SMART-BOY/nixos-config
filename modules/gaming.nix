{ config, lib, pkgs, ... }:

{
  options.mcb.system.enableGaming = lib.mkOption {
    type = lib.types.bool;
    default = true;
    description = "Enable system-level gaming features (Steam, gamemode).";
  };

  config = lib.mkIf config.mcb.system.enableGaming {
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
  };
}
