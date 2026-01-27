{ ... }:

let
  user = "mcbservernixos";
in
{
  imports = [
    ../../profiles/minimal.nix
    ./git.nix
  ];

  home.username = user;
  home.homeDirectory = "/home/${user}";
  home.stateVersion = "25.11";

  programs.home-manager.enable = true;

  xdg.enable = true;
}
