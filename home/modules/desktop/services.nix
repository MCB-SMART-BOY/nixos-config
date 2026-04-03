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
    systemd.user.services.xwaylandvideobridge = lib.mkIf (shared.xwaylandBridgePkg != null) {
      Unit = {
        Description = "XWayland Video Bridge (screen sharing for X11 apps)";
        After = [
          "graphical-session.target"
          "pipewire.service"
          "xdg-desktop-portal.service"
        ];
        PartOf = [ "graphical-session.target" ];
        Wants = [
          "pipewire.service"
          "xdg-desktop-portal.service"
        ];
        ConditionPathExistsGlob = "%t/wayland-*";
      };
      Service = {
        ExecStart = "${shared.xwaylandBridgePkg}/bin/xwaylandvideobridge";
        Restart = "on-failure";
        RestartSec = 2;
      };
      Install = {
        WantedBy = [ "graphical-session.target" ];
      };
    };
  };
}
