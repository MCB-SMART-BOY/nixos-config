{ ... }:

{
  xdg.configFile."niri/config.kdl".source = ./config/niri/config.kdl;
  xdg.configFile."fuzzel/fuzzel.ini".source = ./config/fuzzel/fuzzel.ini;
  xdg.configFile."mako/config".source = ./config/mako/config;
  xdg.configFile."swaylock/config".source = ./config/swaylock/config;
  xdg.configFile."waybar/config".source = ./config/waybar/config;
  xdg.configFile."waybar/style.css".source = ./config/waybar/style.css;
  xdg.configFile."fcitx5/profile".source = ./config/fcitx5/profile;
  xdg.configFile."gtk-2.0/gtkrc".source = ./config/gtk-2.0/gtkrc;
  xdg.configFile."gtk-3.0/settings.ini".source = ./config/gtk-3.0/settings.ini;
  xdg.configFile."gtk-4.0/settings.ini".source = ./config/gtk-4.0/settings.ini;

  home.file.".gtkrc-2.0".source = ./config/gtk-2.0/gtkrc;

  home.file."Pictures/Wallpapers" = {
    source = ./assets/wallpapers;
    recursive = true;
  };

  programs.zsh.initContent = builtins.readFile ./config/zsh/.zshrc;
  programs.tmux.extraConfig = builtins.readFile ./config/tmux/tmux.conf;

  xdg.configFile."starship.toml".source = ./config/starship/starship.toml;
  xdg.configFile."btop/btop.conf".source = ./config/btop/btop.conf;
  xdg.configFile."btop/themes/noctalia.theme".source = ./config/btop/themes/noctalia.theme;
  xdg.configFile."fastfetch/config.jsonc".source = ./config/fastfetch/config.jsonc;

  xdg.configFile."foot/foot.ini".source = ./config/foot/foot.ini;
  xdg.configFile."alacritty/alacritty.toml".source = ./config/alacritty/alacritty.toml;
  xdg.configFile."helix/config.toml".source = ./config/helix/config.toml;
  xdg.configFile."helix/languages.toml".source = ./config/helix/languages.toml;
}
