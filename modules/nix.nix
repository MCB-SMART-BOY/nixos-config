{ mkSourceChecks, pkgs, ... }:

{
  nix = {
    settings = {
      experimental-features = [
        "nix-command"
        "flakes"
      ];
      max-jobs = "auto";
      cores = 0;
      auto-optimise-store = true;
    };

    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 7d";
    };
  };

  system.checks = builtins.attrValues (mkSourceChecks pkgs);

  nixpkgs.config.allowUnfree = true;

  zramSwap.enable = true;
}
