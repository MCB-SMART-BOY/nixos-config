# 包组与开关：集中定义 systemPackages，按功能开关组合。
# 通过 mcb.packages.* 控制不同机器的包集合。
# 新手提示：hosts/profiles/*.nix 里会统一开启/关闭这些组。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  # 读取 mcb.packages.* 开关
  cfg = config.mcb.packages;
  networkCliEnabled = cfg.enableNetwork || cfg.enableNetworkCli;
  networkGuiEnabled = cfg.enableNetwork || cfg.enableNetworkGui;

  resolvePkg =
    path:
    let
      eval =
        if lib.hasAttrByPath path pkgs then
          builtins.tryEval (lib.getAttrFromPath path pkgs)
        else
          {
            success = false;
            value = null;
          };
    in
    if eval.success then eval.value else null;

  pickFirst = list: lib.findFirst (x: x != null) null list;

  obsV4l2sink = pickFirst [
    (resolvePkg [
      "obs-studio-plugins"
      "obs-v4l2sink"
    ])
    (resolvePkg [
      "obs-studio-plugins"
      "v4l2sink"
    ])
  ];

  musicfoxWrapper = pkgs.writeShellScript "musicfox-wrapper" ''
        set -euo pipefail

        musicfox_root="''${MUSICFOX_ROOT:-''${XDG_CONFIG_HOME:-$HOME/.config}/go-musicfox}"
        cfg_file="$musicfox_root/go-musicfox.ini"

        mkdir -p "$musicfox_root"

        # Bootstrap config with sane Linux defaults when no config exists yet.
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
          local file="$1"
          local section="$2"
          local key="$3"
          local value="$4"

          local tmp
          tmp="$(${pkgs.coreutils}/bin/mktemp)"

          ${pkgs.gawk}/bin/awk \
            -v section="$section" \
            -v key="$key" \
            -v value="$value" \
            '
            function print_missing_if_needed() {
              if (in_section && !found_key) {
                print key "=" value
              }
            }

            BEGIN {
              in_section = 0
              section_seen = 0
              found_key = 0
            }

            /^[[:space:]]*\[/ {
              print_missing_if_needed()
              in_section = 0
            }

            {
              line = $0

              if (match(line, /^[[:space:]]*\[([^]]+)\][[:space:]]*$/, m)) {
                if (m[1] == section) {
                  in_section = 1
                  section_seen = 1
                  found_key = 0
                } else {
                  in_section = 0
                }

                print line
                next
              }

              if (in_section && match(line, "^[[:space:]]*" key "[[:space:]]*=")) {
                if (!found_key) {
                  print key "=" value
                  found_key = 1
                }
                next
              }

              print line
            }

            END {
              print_missing_if_needed()

              if (!section_seen) {
                if (NR > 0) {
                  print ""
                }
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

  # Wrap musicfox startup and harden playback-related defaults.
  goMusicfoxCompat = pkgs.runCommand "go-musicfox-compat" { } ''
    mkdir -p "$out/bin"
    install -Dm755 ${musicfoxWrapper} "$out/bin/.musicfox-wrapper"
    ln -s .musicfox-wrapper "$out/bin/musicfox"
    ln -s .musicfox-wrapper "$out/bin/go-musicfox"
  '';

  yesplaymusicPkg =
    if pkgs.stdenv.hostPlatform.system == "x86_64-linux" then
      pkgs.callPackage ../pkgs/yesplaymusic.nix { }
    else
      null;

  baseRuntime = with pkgs; [
    # 基础运行时工具
    bash
    coreutils
    findutils
    gawk
    gnugrep
    iproute2
    procps
    util-linux
    systemd
    # Vulkan 用户态 loader（提供 libvulkan.so.1）
    vulkan-loader
  ];

  networkCli = with pkgs; [
    # 代理核心
    clash-verge-rev
    mihomo
  ];

  networkGui = with pkgs; [
    # 代理 GUI / 面板
    clash-nyanpasu
    metacubexd
    # bluetooth（Noctalia 状态栏 + Blueman 管理界面）
    bluez
    bluez-tools
    blueman
  ];

  shellTools = with pkgs; [
    # 核心命令行工具
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
    # 命令行工作流
    zoxide
    starship
    direnv
    oh-my-zsh
    zsh-autosuggestions
    zsh-syntax-highlighting
    zsh-completions
    fish
    # 监控与磁盘
    btop
    bottom
    fastfetch
    duf
    gdu
    dust
    procs
    # 数据与加密工具
    jq
    yq
    age
    sops
    # 硬件信息
    lm_sensors
    usbutils
    # 终端文件管理
    yazi
  ];

  waylandTools = with pkgs; [
    # 剪贴板与截图
    wl-clipboard
    grim
    slurp
    swappy
    # 通知与壁纸（Noctalia 负责 UI/通知/壁纸）
    libnotify
    # Launcher（Noctalia/GPU 模式菜单会用到）
    fuzzel
    # 会话辅助
    swayidle
    niri
    pipewire
    # 亮度控制
    brightnessctl
  ];

  browsersAndMedia = with pkgs; [
    # 终端
    foot
    alacritty
    # 浏览器
    firefox
    google-chrome
    # 媒体与阅读
    nautilus
    mpv
    vlc
    imv
    zathura
  ];

  dev = with pkgs; [
    # 工具链
    rustup
    # rust-analyzer 通过 rustup 安装（rustup component add rust-analyzer）
    gnumake
    cmake
    pkg-config
    openssl
    gcc
    binutils
    # 编辑器与开发环境
    neovim
    helix
    nodePackages.typescript-language-server
    vscode-fhs
    zed-editor-fhs
    # Python 环境
    uv
    conda
    # 语言服务器与格式化
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

  chat = with pkgs; [
    # 社交聊天
    qq
    telegram-desktop
    discord
  ];

  emulation = with pkgs; [
    # Wine 兼容层
    wineWowPackages.stable
    winetricks
  ];

  entertainment = with pkgs; [
    # 影音与阅读应用（保留占位，避免未来扩展破坏开关结构）
  ];

  office =
    with pkgs;
    [
      # 办公软件
      obsidian
      obs-studio
      libreoffice-still
      xournalpp
    ]
    ++ lib.optionals (obsV4l2sink != null) [ obsV4l2sink ];

  gaming =
    with pkgs;
    [
      # 游戏客户端与工具
      mangohud
      protonup-qt
      lutris
    ]
    ++ lib.optionals (!config.programs.steam.enable) [
      steam
    ];

  systemTools = with pkgs; [
    # 存储与下载
    qbittorrent
    aria2
    yt-dlp
    # 系统工具
    gparted
    pavucontrol
  ];

  insecureTools = with pkgs; [
    # 明确标记为不安全/过时的软件，默认不安装。
    ventoy
  ];

  theming = with pkgs; [
    # 图标、光标与主题
    adwaita-icon-theme
    gnome-themes-extra
    papirus-icon-theme
    bibata-cursors
    catppuccin-gtk
    nwg-look
  ];

  xorgCompat = with pkgs; [
    # Xwayland 兼容
    xwayland
    xwayland-satellite
    xorg.xhost
  ];

  geekTools = with pkgs; [
    # 调试与跟踪
    strace
    ltrace
    gdb
    lldb
    # 二进制工具
    patchelf
    file
    # 性能与监控
    htop
    iotop
    iftop
    sysstat
    lsof
    # 网络诊断
    mtr
    nmap
    tcpdump
    traceroute
    socat
    iperf3
    ethtool
    # 基准测试与分析
    hyperfine
    tokei
    # 压缩与传输
    tree
    unzip
    zip
    p7zip
    rsync
    rclone
    # 构建辅助
    just
    entr
    ncdu
    # 逆向/分析辅助
    binwalk
    radare2
    # 网络抓包
    wireshark
    # Vulkan 诊断
    vulkan-tools
    # 开发协作
    gh
    # 二进制查看
    hexyl
  ];

  life = with pkgs; [
    # 生活类工具
    gnome-calendar
    gnome-clocks
    gnome-calculator
    gnome-weather
    gnome-maps
    gnome-contacts
    baobab
    keepassxc
    simple-scan
  ];

  anime = with pkgs; [
    # 动漫/漫画
    kazumi
    mangayomi
    bilibili
    ani-cli
    mangal
  ];

  music =
    with pkgs;
    [
      # 音乐播放
      goMusicfoxCompat
      ncspot
      mpd
      ncmpcpp
      playerctl
    ]
    ++ lib.optionals (yesplaymusicPkg != null) [ yesplaymusicPkg ];

  # 按开关拼装最终包组
  groups = lib.concatLists [
    baseRuntime
    (lib.optionals networkCliEnabled networkCli)
    (lib.optionals networkGuiEnabled networkGui)
    (lib.optionals cfg.enableShellTools shellTools)
    (lib.optionals cfg.enableWaylandTools waylandTools)
    (lib.optionals cfg.enableBrowsersAndMedia browsersAndMedia)
    (lib.optionals cfg.enableDev dev)
    (lib.optionals cfg.enableChat chat)
    (lib.optionals cfg.enableEmulation emulation)
    (lib.optionals cfg.enableEntertainment entertainment)
    (lib.optionals cfg.enableGaming gaming)
    (lib.optionals cfg.enableSystemTools systemTools)
    (lib.optionals cfg.enableInsecureTools insecureTools)
    (lib.optionals cfg.enableTheming theming)
    (lib.optionals cfg.enableXorgCompat xorgCompat)
    (lib.optionals cfg.enableGeekTools geekTools)
    (lib.optionals cfg.enableOffice office)
    (lib.optionals cfg.enableLife life)
    (lib.optionals cfg.enableAnime anime)
    (lib.optionals cfg.enableMusic music)
  ];
in
{
  options.mcb.packages = {
    # 每个开关控制一个包组，便于不同主机复用
    enableNetwork = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Legacy switch: enable both network CLI and GUI packages.";
    };
    enableNetworkCli = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install network/proxy CLI and service packages.";
    };
    enableNetworkGui = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install network GUI tooling (applets, panels, bluetooth UI).";
    };
    enableShellTools = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install CLI and shell utilities.";
    };
    enableWaylandTools = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install Wayland-related tooling.";
    };
    enableBrowsersAndMedia = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install browsers and media apps.";
    };
    enableDev = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install development toolchain packages.";
    };
    enableChat = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install chat clients.";
    };
    enableEmulation = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install emulation/Wine tooling.";
    };
    enableEntertainment = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install entertainment apps.";
    };
    enableGaming = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install gaming tools.";
    };
    enableSystemTools = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install system utilities.";
    };
    enableInsecureTools = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install insecure/legacy packages (disabled by default).";
    };
    enableTheming = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install theming packages.";
    };
    enableXorgCompat = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install Xorg compatibility tools.";
    };
    enableGeekTools = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install common geek/debug/network tooling.";
    };
    enableOffice = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install office productivity tools.";
    };
    enableLife = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install life/utility desktop apps.";
    };
    enableAnime = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install anime/manga apps.";
    };
    enableMusic = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install music players.";
    };
  };

  config.environment.systemPackages = groups;
}
