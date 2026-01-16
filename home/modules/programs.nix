{ config, lib, ... }:

let
  heavy = config.mcb.packages.enableHeavyBuilds;
in
{
  programs.alacritty = {
    enable = heavy;
  };

  programs.helix = {
    enable = heavy;
  };

  xdg.configFile."foot/foot.ini".source = ../config/foot/foot.ini;
  xdg.configFile."alacritty/alacritty.toml" = lib.mkIf heavy {
    source = ../config/alacritty/alacritty.toml;
  };
  xdg.configFile."helix/config.toml" = lib.mkIf heavy {
    source = ../config/helix/config.toml;
  };
  xdg.configFile."helix/languages.toml" = lib.mkIf heavy {
    source = ../config/helix/languages.toml;
  };
}
