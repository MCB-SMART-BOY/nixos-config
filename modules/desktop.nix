{ pkgs, ... }:

{
  programs.niri.enable = true;
  programs.noctalia.enable = true;
  programs.hyprland.enable = true;

  programs.dconf.enable = true;
  programs.xwayland.enable = true;
  services.xserver = {
    enable = true;
    xkb = {
      layout = "us";
      variant = "";
      options = "ctrl:swapcaps";
    };
  };
  console.useXkbConfig = true;

  services.desktopManager.gnome.enable = true;
  services.displayManager.gdm.enable = true;

  boot.plymouth = {
    enable = true;
    theme = "bgrt";
    themePackages = with pkgs; [ catppuccin-plymouth ];
  };

  environment.sessionVariables = {
    MOZ_ENABLE_WAYLAND = "1";
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    SDL_IM_MODULE = "fcitx";
    GLFW_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
    XIM_SERVERS = "fcitx";
  };

  xdg.portal = {
    enable = true;
    extraPortals = with pkgs; [
      xdg-desktop-portal-gnome
      xdg-desktop-portal-gtk
    ];
    config = {
      common = {
        default = [ "gnome" "gtk" ];
        "org.freedesktop.impl.portal.ScreenCast" = [ "gnome" ];
      };
      niri = {
        default = [ "gnome" "gtk" ];
        "org.freedesktop.impl.portal.ScreenCast" = [ "gnome" ];
      };
    };
  };

  services.gnome = {
    gnome-keyring.enable = true;
    gnome-online-accounts.enable = false;
    gnome-remote-desktop.enable = true;
    rygel.enable = false;
    tinysparql.enable = false;
  };

  environment.gnome.excludePackages = with pkgs; [
    gnome-tour gnome-music gnome-contacts gnome-maps
    gnome-weather epiphany simple-scan totem cheese
    hitori tali iagno
  ];

  programs.geary.enable = false;
}
