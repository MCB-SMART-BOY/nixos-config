# NixOS 25.11 "Xantusia" Configuration
# 最终融合版 - 适配 Niri + Catppuccin + Rust + 中国网络优化
# User: mcbnixos

{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    ./hardware-configuration.nix
  ];

  # ══════════════════════════════════════════════════════════════════
  # 🚀 Boot & Kernel
  # ══════════════════════════════════════════════════════════════════
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;
  boot.kernelPackages = pkgs.linuxPackages_latest;
  boot.kernelParams = [ "mitigations=off" ]; # 性能优先
  boot.kernelModules = [ "kvm-intel" ];

  # ══════════════════════════════════════════════════════════════════
  # 🌐 Networking & Proxy (关键修正)
  # ══════════════════════════════════════════════════════════════════
  networking = {
    hostName = "nixos-dev";
    networkmanager.enable = true;
    firewall = {
      enable = true;
      allowedTCPPorts = [
        22
        2023
        8080
        3000
        4567
      ]; # 4567: suwayomi
    };

    # ✅ 启用系统级代理
    # 这样 Nix 守护进程下载软件时会自动走 Clash (127.0.0.1:7890)
    proxy = {
      default = "http://127.0.0.1:7890";
      noProxy = "127.0.0.1,localhost,internal.domain";
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # ⚙️ Nix Settings (只用官方源 + 代理)
  # ══════════════════════════════════════════════════════════════════
  nix = {
    settings = {
      experimental-features = [
        "nix-command"
        "flakes"
      ];
      auto-optimise-store = true;

      # ⚠️ 删除了所有国内镜像，强制走代理访问全球 CDN (最快且不校验 Hash)
      substituters = [ "https://cache.nixos.org" ];
      trusted-public-keys = [ "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=" ];

      connect-timeout = 20;
      download-attempts = 5;
    };
    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 7d";
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 🌍 Localization & Input
  # ══════════════════════════════════════════════════════════════════
  time.timeZone = "Asia/Shanghai";
  i18n.defaultLocale = "en_US.UTF-8";
  i18n.supportedLocales = [
    "en_US.UTF-8/UTF-8"
    "zh_CN.UTF-8/UTF-8"
  ];

  i18n.inputMethod = {
    enable = true;
    type = "fcitx5";
    fcitx5 = {
      waylandFrontend = true;
      addons = with pkgs; [
        qt6Packages.fcitx5-chinese-addons
        fcitx5-rime
        fcitx5-gtk
        # Catppuccin 皮肤 (如果有包的话，或者手动安装)
      ];
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 🎨 Fonts (解决中文乱码的关键)
  # ══════════════════════════════════════════════════════════════════
  fonts = {
    packages = with pkgs; [
      # 英文/代码
      nerd-fonts.jetbrains-mono
      nerd-fonts.fira-code
      nerd-fonts.iosevka

      # 中文 (Noto 系列是首选)
      noto-fonts-cjk-sans
      noto-fonts-cjk-serif
      source-han-sans
      source-han-serif

      # 图标
      font-awesome
      material-design-icons
    ];

    # 强制指定默认字体，防止 Alacritty 抓瞎
    fontconfig.defaultFonts = {
      monospace = [
        "JetBrainsMono Nerd Font"
        "Noto Sans CJK SC"
      ];
      sansSerif = [ "Noto Sans CJK SC" ];
      serif = [ "Noto Serif CJK SC" ];
      emoji = [ "Noto Color Emoji" ];
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 🪟 Desktop Environment (Niri)
  # ══════════════════════════════════════════════════════════════════
  programs.niri.enable = true;

  # Greetd 登录界面
  services.greetd = {
    enable = true;
    settings = {
      default_session = {
        command = "${pkgs.tuigreet}/bin/tuigreet --time --greeting 'Welcome to NixOS' --asterisks --remember --remember-user-session --cmd niri-session";
        user = "greeter";
      };
    };
  };

  # 修复 tuigreet 权限和日志干扰
  systemd.services.greetd.serviceConfig = {
    Type = "idle";
    StandardInput = "tty";
    StandardOutput = "tty";
    StandardError = "journal";
    TTYReset = true;
    TTYVHangup = true;
    TTYVTDisallocate = true;
  };

  # 🌟 关键：启用 dconf，否则 GTK 主题无法生效
  programs.dconf.enable = true;

  # Wayland 环境变量
  environment.sessionVariables = {
    NIXOS_OZONE_WL = "1";
    MOZ_ENABLE_WAYLAND = "1";
    SDL_VIDEODRIVER = "wayland";
    # 修复 Java GUI (如 JetBrains)
    _JAVA_AWT_WM_NONREPARENTING = "1";
    # GTK/QT
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    SDL_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
  };

  xdg.portal = {
    enable = true;
    wlr.enable = true;
    extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
    config.common.default = "*";
  };

  # ══════════════════════════════════════════════════════════════════
  # 🔧 Programs & Services
  # ══════════════════════════════════════════════════════════════════

  # Clash Verge
  programs.clash-verge = {
    enable = true;
    package = pkgs.clash-verge-rev;
    autoStart = true;
    tunMode = true;
  };
  security.polkit.enable = true;

  # mihomo
  services.mihomo = {
    enable = true;
    configFile = "/etc/mihomo/config.yaml";

    webui = pkgs.metacubexd;
  };

  # daed
  systemd.services.daed = {
    description = "dae dashboard";
    wantedBy = [ "multi-user.target" ];
    after = [ "networt-online.target" ];

    serviceConfig = {
      ExecStart = "${pkgs.daed}/bin/daed run -c /etc/daed";
      Restart = "always";

      User = "root";

      StateDirectory = "daed";
      WorkingDirectory = "/var/lib/daed";
    };
    preStart = ''
      mkdir -p /etc/daed
      if [ ! -f /etc/daed/config.yaml ]; then
        touch /etc/daed/config.yaml
      fi
    '';
  };
  boot.kernel.sysctl = {
    "net.ipv4.ip_forward" = 1;
    "net.ipv4.conf.all.forwarding" = 1;
    "net.ipv6.conf.all.forwarding" = 1;
    "net.ipv6.conf.default.forwarding" = 1;
  };

  programs.nix-ld = {
    enable = true;
    libraries = with pkgs; [
      gtk3
      glib
      gsettings-desktop-schemas
    ];
  }; # 让非 Nix 编译的二进制文件能运行

  services.openssh.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
  };
  services.tlp.enable = true;

  # 硬件加速
  hardware.graphics = {
    enable = true;
    enable32Bit = true;
    extraPackages = with pkgs; [
      intel-media-driver
      libvdpau-va-gl
    ];
  };

  # 虚拟化
  virtualisation = {
    docker = {
      enable = true;
      storageDriver = "overlay2";
    };
    libvirtd.enable = true;
  };
  programs.virt-manager.enable = true;

  # Steam
  programs.steam = {
    enable = true;
    remotePlay.openFirewall = true;
    dedicatedServer.openFirewall = true;
    gamescopeSession.enable = true;
  };
  programs.gamemode.enable = true;

  # git
  programs.git = {
    enable = true;
    lfs.enable = true;
    config = {
      user = {
        name = "MCB-SMART-BOY";
        email = "mcb2720838051@gmail.com";
      };
      pull.rebase = true;
      init.defaultBranch = "master";
      core = {
        quotepath = false;
        editor = "hx";
      };
      color.ui = "auto";

      core.pager = "delta";
      interactive.diffFilter = "delta --color-only";
      delta = {
        navigate = true;
        light = false;
        side-by-side = true;
        line-numbers = true;
      };
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 📦 System Packages (配合 .zshrc)
  # ══════════════════════════════════════════════════════════════════
  environment.systemPackages = with pkgs; [
    # --- 核心 Shell 工具 (.zshrc 依赖) ---
    git
    wget
    curl
    eza # ls 替代
    bat # cat 替代
    ripgrep # grep 替代
    fd # find 替代
    fzf # 模糊搜索
    zoxide # cd 替代
    btop # top 替代
    fastfetch # neofetch 替代
    starship # Prompt
    direnv # 环境管理
    dust # du 的替代品 (直观的磁盘占用饼图，命令是 dust)
    duf # df 的替代品 (可视化的磁盘空间)
    procs # ps 的替代品 (支持高亮和过滤)
    bottom # top 的替代品 (命令是 btm，比 btop 更极客一点，不过 btop 也很好了)

    # --- 文件管理 ---
    yazi # 终端文件管理器
    nautilus # GUI 文件管理器

    # -- web-configuration-tools --
    clash-verge-rev
    clash-nyanpasu
    mihomo
    metacubexd
    daed

    # --- Wayland 桌面组件 ---
    wl-clipboard # 剪贴板
    grim
    slurp
    swappy # 截图全家桶 (配合 Niri config)
    mako # 通知守护进程 (配合 mako config)
    libnotify # 发送通知命令 (notify-send)
    swaybg # 壁纸
    swaylock # 锁屏
    swayidle # 闲置管理
    waybar # 状态栏
    fuzzel # 启动器

    # --- GUI 应用 ---
    alacritty
    foot
    firefox
    google-chrome
    mpv
    vlc
    imv
    zathura

    # --- 主题与美化 ---
    adwaita-icon-theme
    papirus-icon-theme
    bibata-cursors
    # catppuccin-gtk  # 如果 unstable 源里有这个包建议加上，否则手动配置

    # --- 开发 ---
    rustup
    gcc
    clang
    cmake
    pkg-config
    openssl
    helix
    zed-editor
    vscode-fhs
    # LSP
    rust-analyzer
    nil
    marksman
    taplo
    yaml-language-server
    nixfmt-rfc-style
    black
    stylua
    shfmt

    # --- 社交与娱乐 ---
    qq
    telegram-desktop
    wineWowPackages.stable
    winetricks
    kazumi
    mangayomi
    bilibili
    steam
    mangohud
    protonup-qt
    lutris

    # --- 工具 ---
    ventoy
    qbittorrent
    aria2
    yt-dlp
    gparted
    brightnessctl
  ];

  # AppImage 支持
  programs.appimage = {
    enable = true;
    binfmt = true;
  };

  # Shell 配置 (系统级启用 Zsh)
  programs.zsh = {
    enable = true;

    enableCompletion = true;

    autosuggestions.enable = true;
    syntaxHighlighting.enable = true;

    ohMyZsh = {
      enable = true;
      plugins = [
        "git"
        "sudo"
        "docker"
        "rust"
        "fzf"
      ];
      theme = "robbyrussell";
    };
  };

  # 用户
  users.users.mcbnixos = {
    isNormalUser = true;
    description = "mcbnixos";
    extraGroups = [
      "wheel"
      "networkmanager"
      "video"
      "audio"
      "docker"
      "libvirtd"
    ];
    shell = pkgs.zsh;
  };

  nixpkgs.config.allowUnfree = true;
  nixpkgs.config.permittedInsecurePackages = [ "ventoy-1.1.07" ];

  system.stateVersion = "25.11";
}
