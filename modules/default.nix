{ ... }:

{
  imports = [
    ./options.nix
    ./boot.nix
    ./networking.nix
    ./security.nix
    ./nix.nix
    ./packages.nix
    ./i18n.nix
    ./fonts.nix
    ./desktop.nix
    ./services.nix
    ./virtualization.nix
    ./gaming.nix
  ];
}
