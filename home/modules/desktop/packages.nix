{
  pkgs,
  inputs,
  lib,
  ...
}:

let
  shared = import ./shared.nix {
    inherit pkgs inputs;
  };
in
{
  config = {
    home.packages = lib.optionals (shared.xwaylandBridgePkg != null) [ shared.xwaylandBridgePkg ] ++ [
      pkgs.mcbctl
    ];
  };
}
