{ pkgs, ... }:

{
  programs = {
    niri.enable = true;
    noctalia.enable = true;
    hyprland.enable = true;
    dconf.enable = true;
    xwayland.enable = true;
    geary.enable = false;
  };

  services = {
    xserver = {
      enable = true;
      xkb = {
        layout = "us";
        variant = "";
        options = "ctrl:swapcaps";
      };
    };
    desktopManager.gnome.enable = true;
    displayManager.gdm.enable = true;
    gnome = {
      gnome-keyring.enable = true;
      gnome-online-accounts.enable = false;
      gnome-remote-desktop.enable = true;
      rygel.enable = false;
      tinysparql.enable = false;
    };
  };

  console.useXkbConfig = true;

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
        default = [
          "gnome"
          "gtk"
        ];
        "org.freedesktop.impl.portal.ScreenCast" = [ "gnome" ];
      };
      niri = {
        default = [
          "gnome"
          "gtk"
        ];
        "org.freedesktop.impl.portal.ScreenCast" = [ "gnome" ];
      };
    };
  };

  environment.gnome.excludePackages = with pkgs; [
    gnome-tour
    gnome-music
    gnome-contacts
    gnome-maps
    gnome-weather
    epiphany
    simple-scan
    totem
    cheese
    hitori
    tali
    iagno
  ];

}
