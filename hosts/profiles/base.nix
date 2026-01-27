{ ... }:

{
  imports = [
    ../../modules/options.nix
    ../../modules/boot.nix
    ../../modules/networking.nix
    ../../modules/security.nix
    ../../modules/nix.nix
    ../../modules/packages.nix
    ../../modules/services/core.nix
  ];
}
