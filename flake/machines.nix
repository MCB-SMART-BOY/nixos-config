{
  inputs,
  lib,
  mkSourceChecks,
  self,
}:

let
  macDirs = ../machines; # machine directories for short~
  macEts = builtins.readDir macDirs; # machine entries for short~

  macNms = builtins.attrNames (
    # machine names for short~
    lib.filterAttrs (
      name: type: type == "directory" && builtins.pathExists "${macDirs}/${name}/default.nix"
    ) macEts
  );

  macSys =
    name:
    let
      sysFile = "${macDirs}/${name}/system.nix";
    in
    if builtins.pathExists sysFile then
      import sysFile
    else
      throw "machine '${name}' must define ${sysFile}";

  systems = lib.unique (map macSys macNms);
  pkgsBySystem = lib.genAttrs systems (system: import inputs.nixpkgs { inherit system; });

  mkMac =
    machineName:
    let
      macPath = "${macDirs}/${machineName}";
    in
    lib.nameValuePair machineName (
      lib.nixosSystem {
        system = macSys machineName;
        specialArgs = {
          inherit
            inputs
            machineName
            mkSourceChecks
            self
            ;
        };
        modules = [
          macPath
          ../modules
        ];
      }
    );

  nixosConfigurations = builtins.listToAttrs (map mkMac macNms);

  getMacsForSystem = system: lib.filter (name: macSys name == system) macNms;
  mkMacChecks =
    system:
    builtins.listToAttrs (
      map (
        name: lib.nameValuePair "nixos-${name}" nixosConfigurations.${name}.config.system.build.toplevel
      ) (getMacsForSystem system)
    );
in
{
  inherit
    mkMacChecks
    nixosConfigurations
    pkgsBySystem
    systems
    ;
}
