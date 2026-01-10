{ ... }:

{
  imports = [
    ../../nixos/modules
    ./hardware-configuration.nix
  ];

  networking.hostName = "nixos-dev";
  system.stateVersion = "25.11";
}
