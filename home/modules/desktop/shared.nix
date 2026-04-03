{
  pkgs,
  inputs,
}:

let
  defaultNoctaliaSettings = {
    bar = {
      widgets = {
        left = [
          { id = "Launcher"; }
          { id = "Workspace"; }
        ];
        center = [
          { id = "Clock"; }
        ];
        right = [
          { id = "Tray"; }
          { id = "Volume"; }
          { id = "Brightness"; }
          { id = "Battery"; }
          { id = "NotificationHistory"; }
          { id = "ControlCenter"; }
        ];
      };
    };
  };

  unstablePkgs = import inputs.nixpkgs-unstable {
    system = pkgs.stdenv.hostPlatform.system;
    config = pkgs.config;
  };

  legacyPkgs = import inputs.nixpkgs-24_11 {
    system = pkgs.stdenv.hostPlatform.system;
    config = pkgs.config;
  };

  xwaylandBridgePkg =
    let
      stableEval =
        if pkgs ? xwaylandvideobridge then
          builtins.tryEval pkgs.xwaylandvideobridge
        else
          {
            success = false;
            value = null;
          };

      unstableEval =
        if unstablePkgs ? xwaylandvideobridge then
          builtins.tryEval unstablePkgs.xwaylandvideobridge
        else
          {
            success = false;
            value = null;
          };

      legacyEval =
        if legacyPkgs ? xwaylandvideobridge then
          builtins.tryEval legacyPkgs.xwaylandvideobridge
        else
          {
            success = false;
            value = null;
          };
    in
    if stableEval.success then
      stableEval.value
    else if unstableEval.success then
      unstableEval.value
    else if legacyEval.success then
      legacyEval.value
    else
      null;
in
{
  inherit defaultNoctaliaSettings xwaylandBridgePkg;
}
