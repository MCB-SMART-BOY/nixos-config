{ lib, ... }:

{
  imports = [
    # ./options.nix
    ./boot.nix
    ./network.nix
    ./security.nix
    ./nix.nix
    ./packages.nix
    ./i18n.nix
    ./fonts.nix
    ./desktop.nix
    ./virtualisation.nix
    ./game.nix
    ./core.nix
    ./applications.nix
    # ./impermanence.nix
    # ./secrets.nix
  ];
}
