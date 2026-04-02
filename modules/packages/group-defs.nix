# 系统包组定义：按功能组织 systemPackages，供 package module 复用。

{
  config,
  lib,
  pkgs,
}:

let
  cfg = config.mcb.packages;
  mcbctlPkg = pkgs.mcbctl;
  suites = import ./suites.nix {
    inherit pkgs mcbctlPkg;
  };

  networkCliEnabled = cfg.enableNetwork || cfg.enableNetworkCli;
  networkGuiEnabled = cfg.enableNetwork || cfg.enableNetworkGui;

  legacyUserScopedToggles = [
    "enableBrowsersAndMedia"
    "enableDev"
    "enableChat"
    "enableEmulation"
    "enableOffice"
    "enableLife"
    "enableAnime"
    "enableEntertainment"
  ];

  enabledLegacyUserScopedToggles = lib.filter (name: lib.attrByPath [ name ] false cfg) legacyUserScopedToggles;

  baseRuntime = with pkgs; [
    bash
    coreutils
    diffutils
    findutils
    gawk
    gnugrep
    gnused
    gnutar
    gzip
    iproute2
    iputils
    inetutils
    less
    procps
    psmisc
    util-linux
    systemd
    which
    xz
    bzip2
    vulkan-loader
  ];

  networkCli = with pkgs; [
    clash-verge-rev
    mihomo
  ];

  networkGui = with pkgs; [
    clash-nyanpasu
    metacubexd
    bluez
    bluez-tools
    blueman
  ];

  shellTools = with pkgs; [
    git
    lazygit
    wget
    curl
    openssh
    man-db
    man-pages
    bind
    netcat-openbsd
    suites.schedulerCliSuite
    suites.classicAdminSuite
    suites.mailCliSuite
    moreutils
    pciutils
    htop
    lsof
    file
    tree
    unzip
    zip
    p7zip
    rsync
    eza
    fd
    fzf
    ripgrep
    bat
    suites.batExtrasSuite
    delta
    tealdeer
    atuin
    broot
    sd
    xh
    zellij
    ouch
    doggo
    zoxide
    starship
    direnv
    fish
    btop
    bottom
    fastfetch
    duf
    gdu
    dust
    procs
    jq
    yq
    age
    sops
    lm_sensors
    usbutils
    yazi
  ];

  waylandTools = with pkgs; [
    wl-clipboard
    grim
    slurp
    swappy
    libnotify
    fuzzel
    swayidle
    niri
    pipewire
    brightnessctl
  ];

  gaming =
    with pkgs;
    [
      mangohud
      protonup-qt
      lutris
    ]
    ++ lib.optionals (!config.programs.steam.enable) [ steam ];

  systemTools = with pkgs; [
    qbittorrent
    aria2
    yt-dlp
    gparted
    pavucontrol
  ];

  insecureTools = with pkgs; [
    ventoy
  ];

  theming = with pkgs; [
    adwaita-icon-theme
    gnome-themes-extra
    papirus-icon-theme
    bibata-cursors
    catppuccin-gtk
    nwg-look
  ];

  xorgCompat = with pkgs; [
    xwayland
    xwayland-satellite
    xorg.xhost
  ];

  geekTools = with pkgs; [
    strace
    ltrace
    gdb
    lldb
    patchelf
    iotop
    iftop
    sysstat
    mtr
    nmap
    tcpdump
    traceroute
    socat
    iperf3
    ethtool
    hyperfine
    tokei
    rclone
    just
    entr
    ncdu
    binwalk
    radare2
    wireshark
    vulkan-tools
    gh
    hexyl
  ];

  music = with pkgs; [
    suites.goMusicfoxCompat
    ncspot
    mpd
    ncmpcpp
    playerctl
  ];

  groups = lib.concatLists [
    baseRuntime
    (lib.optionals networkCliEnabled networkCli)
    (lib.optionals networkGuiEnabled networkGui)
    (lib.optionals cfg.enableShellTools shellTools)
    (lib.optionals cfg.enableWaylandTools waylandTools)
    (lib.optionals cfg.enableGaming gaming)
    (lib.optionals cfg.enableSystemTools systemTools)
    (lib.optionals cfg.enableInsecureTools insecureTools)
    (lib.optionals cfg.enableTheming theming)
    (lib.optionals cfg.enableXorgCompat xorgCompat)
    (lib.optionals cfg.enableGeekTools geekTools)
    (lib.optionals cfg.enableMusic music)
  ];
in
{
  inherit
    enabledLegacyUserScopedToggles
    groups
    ;
}
