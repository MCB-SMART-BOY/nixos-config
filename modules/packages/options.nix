# 系统包组选项：集中声明 mcb.packages.* 开关。

{ lib, ... }:

let
  mkPackageToggle =
    description:
    lib.mkOption {
      type = lib.types.bool;
      default = false;
      inherit description;
    };

  compatibilityDescription =
    "Compatibility switch kept for host profiles; user apps should be declared explicitly in home/users/<user>/packages.nix.";

  mkCompatibilityToggle = mkPackageToggle compatibilityDescription;
in
{
  options.mcb.packages = {
    enableNetwork = mkPackageToggle "Legacy switch: enable both network CLI and GUI packages.";
    enableNetworkCli = mkPackageToggle "Install network/proxy CLI and service packages.";
    enableNetworkGui = mkPackageToggle "Install network GUI tooling (applets, panels, bluetooth UI).";
    enableShellTools = mkPackageToggle "Install CLI and shell utilities.";
    enableWaylandTools = mkPackageToggle "Install Wayland-related tooling.";
    enableBrowsersAndMedia = mkCompatibilityToggle;
    enableDev = mkCompatibilityToggle;
    enableChat = mkCompatibilityToggle;
    enableEmulation = mkCompatibilityToggle;
    enableEntertainment = mkPackageToggle "Reserved toggle for future entertainment package group.";
    enableGaming = mkPackageToggle "Install gaming tools.";
    enableSystemTools = mkPackageToggle "Install system utilities.";
    enableInsecureTools = mkPackageToggle "Install insecure/legacy packages (disabled by default).";
    enableTheming = mkPackageToggle "Install theming packages.";
    enableXorgCompat = mkPackageToggle "Install Xorg compatibility tools.";
    enableGeekTools = mkPackageToggle "Install common geek/debug/network tooling.";
    enableOffice = mkCompatibilityToggle;
    enableLife = mkCompatibilityToggle;
    enableAnime = mkCompatibilityToggle;
    enableMusic = mkPackageToggle "Install music players.";
  };
}
