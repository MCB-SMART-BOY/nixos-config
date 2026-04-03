{
  config,
  pkgs,
  inputs,
  ...
}:

let
  shared = import ./shared.nix {
    inherit pkgs inputs;
  };
in
{
  config = {
    programs.noctalia-shell.enable = true;
    programs.noctalia-shell.settings =
      if config.mcb.noctalia.barProfile == "default" then
        shared.defaultNoctaliaSettings
      else
        { };
  };
}
