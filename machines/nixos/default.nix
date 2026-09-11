# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{ config, pkgs, ... }:

{
  imports = [
    ./hardware-configuration.nix
    ./hardware-gpu.nix
  ];

  # Define a user account. Don't forget to set a password with ‘passwd’.
  users.users."admin" = {
    isNormalUser = true;
    description = "admin";
    extraGroups = [ "networkmanager" "wheel" "libvirtd" "incus-admin" ];
  };

  programs.google-chrome = {
    enable = true;
  };

  system.stateVersion = "26.05"; # Did you read the comment?
}
