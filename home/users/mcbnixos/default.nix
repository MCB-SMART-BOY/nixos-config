{ ... }:

let
  user = "mcbnixos";
in
{
  imports = [
    ../../profiles/full.nix
    ./git.nix
    ./files.nix
    ./scripts.nix
  ];

  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;

  xdg.enable = true;
}
