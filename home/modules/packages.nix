{ config, lib, pkgs, ... }:

let
  cfg = config.mcb.packages;

  baseRuntime = with pkgs; [
    # Core runtime tools for scripts when the system profile is not on PATH
    bash
    coreutils
    findutils
    gawk
    gnugrep
    iproute2
    procps
    util-linux
    systemd
    pipewire
    niri
    swaybg
  ];

  network = with pkgs; [
    # Proxy core
    clash-verge-rev
    clash-nyanpasu
    mihomo
    metacubexd
    # Network UI
    networkmanagerapplet
  ];

  shellTools = with pkgs; [
    # Core CLI
    git
    wget
    curl
    eza
    fd
    fzf
    # Shell workflow
    zoxide
    starship
    direnv
    fish
    # Monitoring and disk
    btop
    fastfetch
    duf
    gdu
    # Data and crypto tools
    jq
    yq
    age
    sops
    # Hardware info
    lm_sensors
    usbutils
  ];

  waylandTools = with pkgs; [
    # Clipboard and screenshots
    wl-clipboard
    grim
    slurp
    swappy
    # Notifications and wallpaper
    mako
    libnotify
    # Session helpers
    swaylock
    swayidle
    waybar
    fuzzel
    fcitx5
    # Brightness
    brightnessctl
  ];

  browsersAndMedia = with pkgs; [
    # Terminals
    foot
    # Browsers
    firefox
    google-chrome
    # Media and viewers
    nautilus
    mpv
    vlc
    imv
    zathura
  ];

  dev = with pkgs; [
    # Toolchains
    rustup
    # rust-analyzer is managed via rustup (rustup component add rust-analyzer)
    gnumake
    cmake
    pkg-config
    openssl
    # Editors/IDEs
    neovim
    nodePackages.typescript-language-server
    vscode-fhs
    # Python environments
    uv
    conda
    # LSP and formatters
    nil
    marksman
    taplo
    yaml-language-server
    lua-language-server
    gopls
    nixfmt
    black
    stylua
    shfmt
  ];

  heavyBuilds = with pkgs; [
    # Large builds (often compiled from source when cache misses)
    zed
    helix
    alacritty
    yazi
    ripgrep
    bat
    delta
    bottom
    procs
    dust
    # Large toolchains
    # clang
    gcc
  ];

  chat = with pkgs; [
    # Messaging
    qq
    telegram-desktop
    discord
  ];

  emulation = with pkgs; [
    # Wine stack
    wineWowPackages.stable
    winetricks
  ];

  entertainment = with pkgs; [
    # Anime/video apps
    netease-cloud-music-gtk
    kazumi
    mangayomi
    bilibili
  ] ++ lib.optionals (pkgs ? yesplaymusic) [ pkgs.yesplaymusic ]
    ++ lib.optionals (pkgs ? musicfox) [ pkgs.musicfox ]
    ++ lib.optionals (pkgs ? go-musicfox) [ pkgs.go-musicfox ];

  gaming = with pkgs; [
    # Core clients/tools
    steam
    mangohud
    protonup-qt
    lutris
  ];

  systemTools = with pkgs; [
    # Storage and downloads
    ventoy
    qbittorrent
    aria2
    yt-dlp
    # System utilities
    gparted
    pavucontrol
  ];

  theming = with pkgs; [
    # Icons, cursors, themes
    adwaita-icon-theme
    gnome-themes-extra
    papirus-icon-theme
    bibata-cursors
    catppuccin-gtk
    nwg-look
  ];

  xorgCompat = with pkgs; [
    # Xwayland compatibility
    xwayland
    xwayland-satellite
    xorg.xhost
  ];

  geekTools = with pkgs; [
    # Debugging and tracing
    strace
    ltrace
    gdb
    lldb
    # Binary tooling
    binutils
    patchelf
    file
    # Performance and monitoring
    htop
    iotop
    iftop
    sysstat
    lsof
    # Network diagnostics
    mtr
    nmap
    tcpdump
    traceroute
    socat
    iperf3
    ethtool
    # Benchmarking and analysis
    hyperfine
    tokei
    # Archiving and transfer
    tree
    unzip
    zip
    p7zip
    rsync
    rclone
    # Build helpers
    just
    entr
    ncdu
  ];

  groups = lib.concatLists [
    baseRuntime
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
    (lib.optionals cfg.enableGeekTools geekTools)
    (lib.optionals cfg.enableHeavyBuilds heavyBuilds)
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
    enableGeekTools = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install common geek/debug/network tooling.";
    };
    enableHeavyBuilds = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install large packages that may compile from source.";
    };
  };

  config.home.packages = groups;
}
