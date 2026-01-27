{ ... }:

{
  imports = [
    ./base.nix
    ../../modules/i18n.nix
    ../../modules/fonts.nix
    ../../modules/desktop.nix
    ../../modules/services/desktop.nix
    ../../modules/virtualization.nix
    ../../modules/gaming.nix
  ];

  mcb.packages = {
    enableNetwork = true;
    enableShellTools = true;
    enableWaylandTools = true;
    enableBrowsersAndMedia = true;
    enableDev = true;
    enableChat = true;
    enableEmulation = true;
    enableEntertainment = true;
    enableGaming = true;
    enableSystemTools = true;
    enableTheming = true;
    enableXorgCompat = true;
    enableGeekTools = true;
  };
}
