{ inputs, self }:

let
  inherit (inputs.nixpkgs) lib;
  macDirs = ../machines; # machine directories for short~
  macEts = builtins.readDir macDirs; # machine entries for short~

  macNms = builtins.attrNames ( # machine names for short~
    lib.filterAttrs (name: type:
      type == "directory" &&
      builtins.pathExists "${macDirs}/${name}/default.nix"
    ) macEts );

  macSys = name:
    let sysFile = "${macDirs}/${name}/system.nix";
    in
      if builtins.pathExists sysFile
        then import sysFile
        else builtins.currentSystem;

  mkSys = name: lib.nixosSystem {
    specialArgs = { inherit inputs self; }; # I mean ... it looks nothing special yeah?
    modules = [ "${macDirs}/${name}/default.nix" ];
  };

  mkMac = machineName:
    let macPath = "${macDirs}/${machineName}";
    in
      lib.nameValuePair machineName (lib.nixosSystem {
        system = macSys machineName;
        specialArgs = { inherit inputs self machineName; };
        modules = [ macPath ../modules ];
      });

  nixosConfigurations = builtins.listToAttrs (map mkMac macNms);

in {
  inherit nixosConfigurations;
}
