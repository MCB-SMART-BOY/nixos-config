{ inputs, self }:

let
  inherit (inputs.nixpkgs) lib;

  sourceChecks = import ./source-checks.nix { inherit lib self; };
  inherit (sourceChecks) mkSourceChecks;

  machines = import ./machines.nix {
    inherit
      inputs
      lib
      mkSourceChecks
      self
      ;
  };
  inherit (machines)
    mkMacChecks
    nixosConfigurations
    pkgsBySystem
    systems
    ;

  developmentBySystem = lib.genAttrs systems (
    system: import ./development.nix { pkgs = pkgsBySystem.${system}; }
  );

  checks = lib.genAttrs systems (system: mkSourceChecks pkgsBySystem.${system} // mkMacChecks system);
  formatter = lib.mapAttrs (_system: development: development.formatter) developmentBySystem;
  devShells = lib.mapAttrs (_system: development: {
    default = development.devShell;
  }) developmentBySystem;
in
{
  inherit
    checks
    devShells
    formatter
    nixosConfigurations
    ;
}
