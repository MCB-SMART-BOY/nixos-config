# 包组与开关：集中定义 systemPackages，按功能开关组合。
# 选项声明统一在 modules/options.nix（mcb.packages.*）；游戏包跟随 programs.steam.enable）。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.mcb.packages;

  # ── musicfox 兼容包装 ─────────────────────────────────────────
  musicfoxWrapper = pkgs.writeShellScript "musicfox-wrapper" ''
    set -euo pipefail

    musicfox_root="''${MUSICFOX_ROOT:-''${XDG_CONFIG_HOME:-$HOME/.config}/go-musicfox}"
    cfg_file="$musicfox_root/go-musicfox.ini"

    mkdir -p "$musicfox_root"

    if [ ! -f "$cfg_file" ]; then
      cat > "$cfg_file" <<CFG
    [player]
    engine=mpv
    mpvBin=${pkgs.mpv}/bin/mpv

    [unm]
    switch=true
    sources=kuwo,kugou,migu,qq
    searchLimit=3
    skipInvalidTracks=true
    CFG
    fi

    ensure_ini_key() {
      local file="$1" section="$2" key="$3" value="$4"
      local tmp
      tmp="$(${pkgs.coreutils}/bin/mktemp)"

      ${pkgs.gawk}/bin/awk \
        -v section="$section" \
        -v key="$key" \
        -v value="$value" \
        '
        function print_missing_if_needed() {
          if (in_section && !found_key) { print key "=" value }
        }
        BEGIN { in_section = 0; section_seen = 0; found_key = 0 }
        /^[[:space:]]*\[/ { print_missing_if_needed(); in_section = 0 }
        {
          line = $0
          if (match(line, /^[[:space:]]*\[([^]]+)\][[:space:]]*$/, m)) {
            if (m[1] == section) { in_section = 1; section_seen = 1; found_key = 0 }
            else { in_section = 0 }
            print line; next
          }
          if (in_section && match(line, "^[[:space:]]*" key "[[:space:]]*=")) {
            if (!found_key) { print key "=" value; found_key = 1 }
            next
          }
          print line
        }
        END {
          print_missing_if_needed()
          if (!section_seen) {
            if (NR > 0) print ""
            print "[" section "]"
            print key "=" value
          }
        }
        ' "$file" > "$tmp"

      ${pkgs.coreutils}/bin/mv "$tmp" "$file"
    }

    ensure_ini_key "$cfg_file" "player" "engine" "mpv"
    ensure_ini_key "$cfg_file" "player" "mpvBin" "${pkgs.mpv}/bin/mpv"
    ensure_ini_key "$cfg_file" "unm" "switch" "true"
    ensure_ini_key "$cfg_file" "unm" "sources" "kuwo,kugou,migu,qq"
    ensure_ini_key "$cfg_file" "unm" "searchLimit" "3"
    ensure_ini_key "$cfg_file" "unm" "skipInvalidTracks" "true"

    export MUSICFOX_ROOT="$musicfox_root"
    exec ${pkgs.go-musicfox}/bin/musicfox "$@"
  '';

  goMusicfoxCompat = pkgs.runCommand "go-musicfox-compat" { } ''
    mkdir -p "$out/bin"
    install -Dm755 ${musicfoxWrapper} "$out/bin/.musicfox-wrapper"
    ln -s .musicfox-wrapper "$out/bin/musicfox"
    ln -s .musicfox-wrapper "$out/bin/go-musicfox"
  '';

  # ── 包组定义 ──────────────────────────────────────────────────
  baseRuntime = with pkgs; [
    bash
    coreutils
    findutils
    gawk
    gnugrep
    iproute2
    procps
    util-linux
    systemd
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
    eza
    fd
    fzf
    ripgrep
    bat
    delta
    zoxide
    starship
    direnv
    oh-my-zsh
    zsh-autosuggestions
    zsh-syntax-highlighting
    zsh-completions
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
    # ventoy  # 取消注释以安装（标记为不安全）
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
    xhost
  ];

  geekTools = with pkgs; [
    strace
    ltrace
    gdb
    lldb
    patchelf
    file
    htop
    iotop
    iftop
    sysstat
    lsof
    mtr
    nmap
    tcpdump
    traceroute
    socat
    iperf3
    ethtool
    hyperfine
    tokei
    tree
    unzip
    zip
    p7zip
    rsync
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
    goMusicfoxCompat
    ncspot
    mpd
    ncmpcpp
    playerctl
  ];

  groups = lib.concatLists [
    baseRuntime
    (lib.optionals cfg.enableNetworkCli networkCli)
    (lib.optionals cfg.enableNetworkGui networkGui)
    (lib.optionals cfg.enableShellTools shellTools)
    (lib.optionals cfg.enableWaylandTools waylandTools)
    (lib.optionals config.programs.steam.enable gaming)
    (lib.optionals cfg.enableSystemTools systemTools)
    (lib.optionals cfg.enableTheming theming)
    (lib.optionals cfg.enableXorgCompat xorgCompat)
    (lib.optionals cfg.enableGeekTools geekTools)
    (lib.optionals cfg.enableMusic music)
  ];
in
{
  config = {
    environment.systemPackages = groups;
  };
}
