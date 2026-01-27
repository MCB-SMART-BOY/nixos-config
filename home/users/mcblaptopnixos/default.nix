{ ... }:

let
  user = "mcblaptopnixos";
in
{
  imports = [
    ../../profiles/full.nix
    ./git.nix
  ];

  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;

  xdg.enable = true;
}
