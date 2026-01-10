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

  programs.swaylock.enable = true;
  programs.fuzzel.enable = true;
  programs.waybar.enable = true;
}
