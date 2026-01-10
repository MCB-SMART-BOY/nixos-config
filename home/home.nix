{ vars, ... }:

let
  user = vars.user;
in
{
  imports = [
    ./modules/base.nix
    ./modules/packages.nix
    ./modules/programs.nix
    ./modules/desktop.nix
    ./modules/shell.nix
    ./modules/git.nix
  ];

  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;

  xdg.enable = true;
}
