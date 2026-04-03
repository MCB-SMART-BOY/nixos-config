{
  inputs,
  ...
}:

{
  imports = [
    inputs.noctalia.homeModules.default
    ./desktop/options.nix
    ./desktop/noctalia.nix
    ./desktop/session.nix
    ./desktop/packages.nix
    ./desktop/mime.nix
    ./desktop/entries.nix
    ./desktop/services.nix
  ];
}
