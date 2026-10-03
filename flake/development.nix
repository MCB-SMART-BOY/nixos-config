{ pkgs }:

let
  formatter = pkgs.nixfmt-tree.override {
    settings = {
      tree-root-file = "flake.nix";
      formatter.nixfmt.excludes = [ "machines/*/hardware-configuration.nix" ];
    };
  };
in
{
  inherit formatter;

  devShell = pkgs.mkShellNoCC {
    packages = with pkgs; [
      deadnix
      gitleaks
      formatter
      statix
      trivy
      vulnix
    ];
  };
}
