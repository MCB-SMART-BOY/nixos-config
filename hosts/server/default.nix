{ config, lib, pkgs, ... }:

let
  hardwareFile =
    if builtins.pathExists ./hardware-configuration.nix then
      ./hardware-configuration.nix
    else if builtins.pathExists ../../hardware-configuration.nix then
      ../../hardware-configuration.nix
    else
      null;
  allUsers =
    if config.mcb.users != [ ] then
      config.mcb.users
    else
      [ config.mcb.user ];
in
{
  imports =
    [ ../profiles/server.nix ]
    ++ lib.optional (hardwareFile != null) hardwareFile
    ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  mcb = {
    user = "mcbservernixos";
    users = [ "mcbservernixos" ];
    cpuVendor = "intel";
    proxyMode = "off";
  };

  networking.hostName = "server";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  users.users = lib.genAttrs allUsers (name: {
    isNormalUser = true;
    description = name;
    extraGroups = [
      "wheel"
      "networkmanager"
      "docker"
      "libvirtd"
    ];
    shell = pkgs.zsh;
    linger = true;
  });
}
