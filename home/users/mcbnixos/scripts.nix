{ pkgs, ... }:

let
  mkScript = { name, runtimeInputs ? [ ] }:
    pkgs.writeShellApplication {
      inherit name runtimeInputs;
      text = builtins.readFile ./scripts/${name};
    };

  scripts = {
    lock-screen = mkScript {
      name = "lock-screen";
      runtimeInputs = [ pkgs.swaylock ];
    };

    niri-run = mkScript {
      name = "niri-run";
    };

    wallpaper-random = mkScript {
      name = "wallpaper-random";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.findutils
        pkgs.procps
        pkgs.systemd
        pkgs.swaybg
      ];
    };

    waybar-flake-updates = mkScript {
      name = "waybar-flake-updates";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.gawk
        pkgs.git
        pkgs.jq
        pkgs.util-linux
      ];
    };

    waybar-net-speed = mkScript {
      name = "waybar-net-speed";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.gawk
        pkgs.gnugrep
        pkgs.iproute2
      ];
    };

    waybar-proxy-status = mkScript {
      name = "waybar-proxy-status";
      runtimeInputs = [ pkgs.systemd ];
    };
  };

  mkBinLink = name: {
    source = "${scripts.${name}}/bin/${name}";
  };
in
{
  home.packages = builtins.attrValues scripts;

  home.file.".local/bin/lock-screen" = mkBinLink "lock-screen";
  home.file.".local/bin/niri-run" = mkBinLink "niri-run";
  home.file.".local/bin/wallpaper-random" = mkBinLink "wallpaper-random";
  home.file.".local/bin/waybar-flake-updates" = mkBinLink "waybar-flake-updates";
  home.file.".local/bin/waybar-net-speed" = mkBinLink "waybar-net-speed";
  home.file.".local/bin/waybar-proxy-status" = mkBinLink "waybar-proxy-status";

  systemd.user.services.wallpaper-random = {
    Unit = {
      Description = "Random wallpaper (swaybg)";
      After = [ "graphical-session.target" ];
      PartOf = [ "graphical-session.target" ];
      ConditionPathExistsGlob = "%t/wayland-*";
    };
    Service = {
      Type = "oneshot";
      ExecStart = "%h/.local/bin/wallpaper-random";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };

  systemd.user.timers.wallpaper-random = {
    Unit = {
      Description = "Rotate wallpaper periodically";
      PartOf = [ "graphical-session.target" ];
    };
    Timer = {
      OnBootSec = "1m";
      OnUnitActiveSec = "10m";
      AccuracySec = "1m";
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
