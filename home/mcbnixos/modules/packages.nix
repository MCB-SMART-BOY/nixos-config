{ config, lib, pkgs, ... }:

let
  cfg = config.mcb.packages;

  network = with pkgs; [
    clash-verge-rev
    mihomo
    metacubexd
  ];

  shellTools = with pkgs; [
    git
    wget
    curl
    eza
    bat
    ripgrep
    fd
    btop
    fastfetch
    dust
    duf
    procs
    bottom
    delta
    gdu
    jq
    yq
    age
    sops
    lm_sensors
    usbutils
  ];

  waylandTools = with pkgs; [
    wl-clipboard
    grim
    slurp
    swappy
    libnotify
    swaybg
    swayidle
    brightnessctl
  ];

  browsersAndMedia = with pkgs; [
    foot
    firefox
    google-chrome
    mpv
    vlc
    imv
    zathura
  ];

  dev = with pkgs; [
    rustup
    gcc
    clang
    cmake
    pkg-config
    openssl
    nodejs
    nodePackages.bash-language-server
    nodePackages.pyright
    nodePackages.typescript-language-server
    zed-editor
    vscode-fhs
    rust-analyzer
    nil
    marksman
    taplo
    yaml-language-server
    lua-language-server
    gopls
    nixfmt-rfc-style
    black
    stylua
    shfmt
  ];

  chat = with pkgs; [
    qq
    telegram-desktop
    discord
  ];

  emulation = with pkgs; [
    wineWowPackages.stable
    winetricks
  ];

  entertainment = with pkgs; [
    kazumi
    mangayomi
    bilibili
  ];

  gaming = with pkgs; [
    mangohud
    protonup-qt
    lutris
  ];

  systemTools = with pkgs; [
    ventoy
    qbittorrent
    aria2
    yt-dlp
    gparted
  ];

  theming = with pkgs; [
    adwaita-icon-theme
    papirus-icon-theme
    bibata-cursors
    catppuccin-gtk
    nwg-look
  ];

  xorgCompat = with pkgs; [
    xwayland-satellite
    xorg.xhost
  ];

  groups = lib.concatLists [
    (lib.optionals cfg.enableNetwork network)
    (lib.optionals cfg.enableShellTools shellTools)
    (lib.optionals cfg.enableWaylandTools waylandTools)
    (lib.optionals cfg.enableBrowsersAndMedia browsersAndMedia)
    (lib.optionals cfg.enableDev dev)
    (lib.optionals cfg.enableChat chat)
    (lib.optionals cfg.enableEmulation emulation)
    (lib.optionals cfg.enableEntertainment entertainment)
    (lib.optionals cfg.enableGaming gaming)
    (lib.optionals cfg.enableSystemTools systemTools)
    (lib.optionals cfg.enableTheming theming)
    (lib.optionals cfg.enableXorgCompat xorgCompat)
  ];
in
{
  options.mcb.packages = {
    enableNetwork = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install network/proxy tooling.";
    };
    enableShellTools = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install CLI and shell utilities.";
    };
    enableWaylandTools = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install Wayland-related tooling.";
    };
    enableBrowsersAndMedia = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install browsers and media apps.";
    };
    enableDev = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install development toolchain packages.";
    };
    enableChat = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install chat clients.";
    };
    enableEmulation = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install emulation/Wine tooling.";
    };
    enableEntertainment = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install entertainment apps.";
    };
    enableGaming = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install gaming tools.";
    };
    enableSystemTools = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install system utilities.";
    };
    enableTheming = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install theming packages.";
    };
    enableXorgCompat = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install Xorg compatibility tools.";
    };
  };

  config.home.packages = groups;
}
