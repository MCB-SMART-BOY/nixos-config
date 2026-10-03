{ lib, self }:

let
  isBuildResult = path: type: type == "symlink" && toString path == "${self.outPath}/result";
  isGeneratedHardwareConfig =
    path: builtins.match ".*/machines/[^/]+/hardware-configuration\\.nix" (toString path) != null;

  projectSource = lib.cleanSourceWith {
    src = self.outPath;
    filter = path: type: !(isBuildResult path type);
  };

  checkedSource = lib.cleanSourceWith {
    src = projectSource;
    filter = path: _type: !(isGeneratedHardwareConfig path);
  };

  mkSourceCheck =
    pkgs:
    {
      checkName,
      checkSource,
      runtimeInputs,
      script,
    }:
    pkgs.stdenvNoCC.mkDerivation {
      name = "source-${checkName}";
      builder = "${pkgs.bash}/bin/bash";
      args = [
        ../scripts/run-source-check.sh
        script
      ];
      PATH = lib.makeBinPath (
        [
          pkgs.bash
          pkgs.coreutils
        ]
        ++ runtimeInputs
      );
      inherit checkName checkSource;
    };

  mkSourceChecks = pkgs: {
    format = mkSourceCheck pkgs {
      checkName = "format";
      checkSource = checkedSource;
      runtimeInputs = [
        pkgs.coreutils
        pkgs.findutils
        pkgs.nixfmt
      ];
      script = ../scripts/check-format.sh;
    };

    statix = mkSourceCheck pkgs {
      checkName = "statix";
      checkSource = checkedSource;
      runtimeInputs = [ pkgs.statix ];
      script = ../scripts/check-statix.sh;
    };

    deadnix = mkSourceCheck pkgs {
      checkName = "deadnix";
      checkSource = checkedSource;
      runtimeInputs = [ pkgs.deadnix ];
      script = ../scripts/check-deadnix.sh;
    };

    secrets = mkSourceCheck pkgs {
      checkName = "secrets";
      checkSource = projectSource;
      runtimeInputs = [ pkgs.gitleaks ];
      script = ../scripts/check-secrets.sh;
    };
  };
in
{
  inherit mkSourceChecks;
}
