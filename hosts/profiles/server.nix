{ ... }:

{
  imports = [
    ./base.nix
  ];

  mcb.packages = {
    enableNetwork = true;
    enableShellTools = true;
    enableWaylandTools = false;
    enableBrowsersAndMedia = false;
    enableDev = false;
    enableChat = false;
    enableEmulation = false;
    enableEntertainment = false;
    enableGaming = false;
    enableSystemTools = true;
    enableTheming = false;
    enableXorgCompat = false;
    enableGeekTools = false;
  };
}
