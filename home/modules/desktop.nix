{ ... }:

{
  xdg.configFile."niri/config.kdl".source = ../config/niri/config.kdl;
  xdg.configFile."fuzzel/fuzzel.ini".source = ../config/fuzzel/fuzzel.ini;
  xdg.configFile."mako/config".source = ../config/mako/config;
  xdg.configFile."swaylock/config".source = ../config/swaylock/config;
  xdg.configFile."waybar/config".source = ../config/waybar/config;
  xdg.configFile."waybar/style.css".source = ../config/waybar/style.css;
  xdg.configFile."gtk-3.0/settings.ini".source = ../config/gtk-3.0/settings.ini;
  xdg.configFile."gtk-4.0/settings.ini".source = ../config/gtk-4.0/settings.ini;

  home.file."Pictures/Wallpapers" = {
    source = ../assets/wallpapers;
    recursive = true;
  };

  home.file.".local/bin/wallpaper-random" = {
    source = ../scripts/wallpaper-random;
    executable = true;
  };

  home.file.".local/bin/lock-screen" = {
    source = ../scripts/lock-screen;
    executable = true;
  };

  home.file.".local/bin/niri-run" = {
    source = ../scripts/niri-run;
    executable = true;
  };

  home.file.".local/bin/waybar-flake-updates" = {
    source = ../scripts/waybar-flake-updates;
    executable = true;
  };

  home.file.".local/bin/waybar-proxy-status" = {
    source = ../scripts/waybar-proxy-status;
    executable = true;
  };

  home.file.".local/bin/waybar-net-speed" = {
    source = ../scripts/waybar-net-speed;
    executable = true;
  };

  systemd.user.services.wallpaper-random = {
    Unit = {
      Description = "Random wallpaper (swaybg)";
      After = [ "graphical-session.target" ];
      PartOf = [ "graphical-session.target" ];
    };
    Service = {
      Type = "simple";
      ExecStart = "%h/.local/bin/wallpaper-random";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };

  programs.swaylock.enable = true;
  programs.fuzzel.enable = true;
  programs.waybar.enable = true;
}
