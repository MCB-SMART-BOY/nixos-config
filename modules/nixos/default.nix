{ ... }:

{
  imports = [
    ./boot.nix
    ./networking.nix
    ./security.nix
    ./nix.nix
    ./i18n.nix
    ./fonts.nix
    ./desktop.nix
    ./services.nix
    ./virtualization.nix
    ./gaming.nix
    ./users.nix
    ./system-packages.nix
  ];
}
