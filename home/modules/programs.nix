{ ... }:

{
  programs.alacritty = {
    enable = true;
  };

  programs.helix = {
    enable = true;
  };

  xdg.configFile."alacritty/alacritty.toml".source = ../config/alacritty/alacritty.toml;
  xdg.configFile."helix/config.toml".source = ../config/helix/config.toml;
  xdg.configFile."helix/languages.toml".source = ../config/helix/languages.toml;
}
