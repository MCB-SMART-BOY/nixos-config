# 包组与开关：集中定义 systemPackages，按功能开关组合。
# 通过 mcb.packages.* 控制不同机器的包集合。
# 新手提示：hosts/profiles/*.nix 里会统一开启/关闭这些组。

{ config, lib, pkgs, ... }:

let
  # 读取 mcb.packages.* 开关
  cfg = config.mcb.packages;

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
  ];

  network = with pkgs; [
    # 代理核心
    clash-verge-rev
    clash-nyanpasu
    mihomo
    metacubexd
    # 网络界面
    networkmanagerapplet
    # bluetooth
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
    # 通知与壁纸
    mako
    libnotify
    # 会话辅助
    swaylock
    swayidle
    waybar
    fuzzel
    niri
    swaybg
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

  office = with pkgs; [
    # 办公软件
    libreoffice-still
    wps-office
    xournalpp
  ];

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
    # 逆向/分析辅助
    binwalk
    radare2
    # 网络抓包
    wireshark
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

  music = with pkgs; [
    # 音乐播放
    go-musicfox
    ncspot
    mpd
    ncmpcpp
    playerctl
  ];

  # 按开关拼装最终包组
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
      description = "Install network/proxy tooling.";
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
