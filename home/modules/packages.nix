{ config, lib, pkgs, osConfig ? null, ... }:

let
  cfg = config.mcb.packages;

  baseRuntime = with pkgs; [
    # 系统环境不在 PATH 时也可用的脚本运行基础工具
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
    # 代理核心
    clash-verge-rev
    clash-nyanpasu
    mihomo
    metacubexd
    # 网络界面
    networkmanagerapplet
  ];

  shellTools = with pkgs; [
    # 核心命令行工具
    git
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
    # 通知与壁纸
    mako
    libnotify
    # 会话辅助
    swaylock
    swayidle
    waybar
    fuzzel
    fcitx5
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
    # 影音与阅读应用
    netease-cloud-music-gtk
    kazumi
    mangayomi
    bilibili
  ] ++ lib.optionals (pkgs ? yesplaymusic) [ pkgs.yesplaymusic ]
    ++ lib.optionals (pkgs ? musicfox) [ pkgs.musicfox ]
    ++ lib.optionals (pkgs ? go-musicfox) [ pkgs.go-musicfox ];

  gaming =
    with pkgs;
    [
      # 游戏客户端与工具
      mangohud
      protonup-qt
      lutris
    ]
    ++ lib.optionals (osConfig == null || !(osConfig.programs.steam.enable or false)) [
      steam
    ];

  systemTools = with pkgs; [
    # 存储与下载
    ventoy
    qbittorrent
    aria2
    yt-dlp
    # 系统工具
    gparted
    pavucontrol
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
  };

  config.home.packages = groups;
}
