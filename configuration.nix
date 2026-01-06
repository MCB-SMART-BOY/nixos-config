# ══════════════════════════════════════════════════════════════════════
# ❄️ NixOS 25.11 "Xantusia" Configuration (Verified Ultimate Edition)
# ══════════════════════════════════════════════════════════════════════
# 👤 用户: mcbnixos
# 🛠️ 核心架构: Niri (Wayland) + Clash Verge Rev (Service Mode)
# ✅ 验证状态:
#    1. 已确认 clash-verge-service 二进制存在，采用 System Service 方案。
#    2. 网络栈采用 "loose" 模式，完美兼容 TUN/Docker。
#    3. 字体配置已修正为正确的 Family Name。
# ══════════════════════════════════════════════════════════════════════

{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [ ./hardware-configuration.nix ];

  # ══════════════════════════════════════════════════════════════════
  # 🚀 引导与内核 (Boot & Kernel)
  # ══════════════════════════════════════════════════════════════════
  boot = {
    loader = {
      systemd-boot = {
        enable = true;
        editor = false; # 🔒 安全：禁止在启动菜单修改内核参数
        configurationLimit = 10;
      };
      efi.canTouchEfiVariables = true;
    };

    kernelPackages = pkgs.linuxPackages_latest;
    kernelModules = [
      "kvm-intel"
      "tun"
    ];

    # 🌐 网络栈深度调优 (BBR + Forwarding)
    kernel.sysctl = {
      "net.core.default_qdisc" = "fq";
      "net.ipv4.tcp_congestion_control" = "bbr";

      # 🔥 开启转发 (Docker & Clash TUN 必需)
      "net.ipv4.ip_forward" = 1;
      "net.ipv6.conf.all.forwarding" = 1;

      # 🚀 高并发优化 (防止 BT/P2P 断流)
      "net.netfilter.nf_conntrack_max" = 131072;
      "net.netfilter.nf_conntrack_tcp_timeout_established" = 1200;
      "net.core.rmem_max" = 16777216;
      "net.core.wmem_max" = 16777216;
      "net.ipv4.tcp_rmem" = "4096 87380 16777216";
      "net.ipv4.tcp_wmem" = "4096 65536 16777216";
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 🌐 网络与防火墙 (Networking)
  # ══════════════════════════════════════════════════════════════════
  networking = {
    hostName = "nixos-dev";

    networkmanager = {
      enable = true;
      dns = "none"; # 🚫 让 NM 停止管理 DNS，防止覆盖 /etc/resolv.conf
    };

    # 🛡️ 静态 DNS (本地优先)
    # 逻辑：请求 -> 127.0.0.1 (Clash) -> 失败则走 223.5.5.5
    nameservers = [
      "127.0.0.1"
      "223.5.5.5"
      "1.1.1.1"
    ];

    firewall = {
      enable = true;
      # ✅ 采用 "loose" 模式：允许 TUN 流量返回，同时阻止 IP 欺骗
      checkReversePath = "loose";

      allowedTCPPorts = [
        22
        7890
        9090
      ];
      allowedUDPPorts = [ 53 ];

      # 🤝 信任接口 (Docker/VM/TUN)
      trustedInterfaces = [
        "clash0" # CVR TUN
        "utun+" # 其他 VPN
        "docker0" # Docker Bridge
        "virbr0" # KVM Bridge
      ];
    };
  };

  services.resolved.enable = false; # 避免占用 53 端口

  # ══════════════════════════════════════════════════════════════════
  # 🛡️ Clash Verge Rev 服务 (System Service Mode)
  # ══════════════════════════════════════════════════════════════════

  # [技术说明]
  # 我们在此手动定义 "Service Mode" 守护进程。
  # 这比让 GUI 通过 Polkit 提权更稳定，且符合 NixOS 声明式哲学。
  # 启动后，GUI 设置里的 "Service Mode" 会自动检测为 Active。

  systemd.services.clash-verge-service = {
    description = "Clash Verge Service Mode Daemon";
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "simple";
      Restart = "always";
      # ✅ 已验证：Nixpkgs 的 clash-verge-rev 包含此二进制
      ExecStart = "${pkgs.clash-verge-rev}/bin/clash-verge-service";

      # 最小权限原则 (Capabilities)
      CapabilityBoundingSet = [
        "CAP_NET_ADMIN"
        "CAP_NET_BIND_SERVICE"
        "CAP_NET_RAW"
      ];
      AmbientCapabilities = [
        "CAP_NET_ADMIN"
        "CAP_NET_BIND_SERVICE"
        "CAP_NET_RAW"
      ];
    };
  };

  systemd.services.nix-daemon.environment = {
    https_proxy = "https://localhost:7890";
    http_proxy = "https://localhost:7890";
  };

  # ══════════════════════════════════════════════════════════════════
  # ⏸️ Mihomo 备用配置 (Fallback)
  # ══════════════════════════════════════════════════════════════════

  # ✅ 路径修正：生成 /etc/mihomo/config.yaml
  # environment.etc."mihomo/config.yaml".source = /etc/mihomo/config.yaml;

  services.mihomo = {
    enable = false; # 🚫 默认禁用，作为备胎
    configFile = "/etc/mihomo/config.yaml";
  };

  systemd.services.mihomo = {
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];
    serviceConfig = {
      User = "root";
      Group = "root";
      CapabilityBoundingSet = [
        "CAP_NET_ADMIN"
        "CAP_NET_BIND_SERVICE"
        "CAP_NET_RAW"
      ];
      AmbientCapabilities = [
        "CAP_NET_ADMIN"
        "CAP_NET_BIND_SERVICE"
        "CAP_NET_RAW"
      ];
      WorkingDirectory = "/var/lib/mihomo";
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 🔐 权限与 Polkit (Security)
  # ══════════════════════════════════════════════════════════════════
  security.polkit = {
    enable = true;
    # 允许 Wheel 组管理网络 (GUI 切换代理时可能需要)
    extraConfig = ''
      polkit.addRule(function(action, subject) {
        if (action.id.indexOf("org.freedesktop.NetworkManager.") == 0 && subject.isInGroup("wheel")) {
          return polkit.Result.YES;
        }
      });
    '';
  };

  # ══════════════════════════════════════════════════════════════════
  # ⚙️ Nix 与软件包 (Packages)
  # ══════════════════════════════════════════════════════════════════
  nix = {
    settings = {
      experimental-features = [
        "nix-command"
        "flakes"
      ];
      auto-optimise-store = true;
      # substituters = [
      # "https://mirrors.ustc.edu.cn/nix-channels/store"
      # "https://mirrors.sjtu.edu.cn/nix-channels/store"
      # "https://cache.nixos.org"
      # ];
      # trusted-public-keys = [
      # "mirror.sjtu.edu.cn-nix-channels:5XZJVLcUYq3pP8+8aGM3jLLywiDg9cL8Lp3kVqL3bBk="
      # "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      # ];
    };
    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 7d";
    };
  };

  nixpkgs.config.allowUnfree = true;
  nixpkgs.config.permittedInsecurePackages = [ "ventoy-1.1.07" ];

  # ══════════════════════════════════════════════════════════════════
  # 🌍 本地化与输入法 (I18n)
  # ══════════════════════════════════════════════════════════════════
  time.timeZone = "Asia/Shanghai";
  i18n = {
    defaultLocale = "en_US.UTF-8";
    supportedLocales = [
      "en_US.UTF-8/UTF-8"
      "zh_CN.UTF-8/UTF-8"
    ];
    inputMethod = {
      enable = true;
      type = "fcitx5";
      fcitx5 = {
        waylandFrontend = true;
        addons = with pkgs; [
          qt6Packages.fcitx5-chinese-addons
          fcitx5-rime
          fcitx5-gtk
          qt6Packages.fcitx5-configtool
        ];
      };
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 🎨 字体配置 (Fonts)
  # ══════════════════════════════════════════════════════════════════
  fonts = {
    packages = with pkgs; [
      nerd-fonts.jetbrains-mono
      nerd-fonts.fira-code
      noto-fonts-cjk-sans
      noto-fonts-cjk-serif
      source-han-sans
      source-han-serif
      lxgw-wenkai # ✅ 霞鹜文楷 (包含 Mono 版)
      font-awesome
      wqy_zenhei
      wqy_microhei
    ];
    fontconfig = {
      defaultFonts = {
        # ✅ 修正：等宽字体优先使用 "LXGW WenKai Mono"
        monospace = [
          "JetBrainsMono Nerd Font"
          "LXGW WenKai Mono"
        ];
        # ✅ 修正：无衬线字体使用 "LXGW WenKai"
        sansSerif = [
          "Noto Sans CJK SC"
          "LXGW WenKai"
        ];
        serif = [
          "Noto Serif CJK SC"
          "Source Han Serif SC"
        ];
        emoji = [ "Noto Color Emoji" ];
      };
      antialias = true;
      hinting.enable = true;
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 🪟 Niri 桌面环境 (Desktop)
  # ══════════════════════════════════════════════════════════════════
  programs.niri.enable = true;
  programs.dconf.enable = true;
  programs.xwayland.enable = true;

  services.greetd = {
    enable = true;
    settings.default_session = {
      command = "${pkgs.tuigreet}/bin/tuigreet --time --greeting 'Welcome to NixOS' --asterisks --remember --remember-user-session --cmd niri-session";
      user = "greeter";
    };
  };

  systemd.services.greetd.serviceConfig = {
    Type = "idle";
    StandardInput = "tty";
    StandardOutput = "tty";
    StandardError = "journal";
    TTYReset = true;
    TTYVHangup = true;
    TTYVTDisallocate = true;
  };

  # 🚀 强制 Wayland 模式
  environment.sessionVariables = {
    NIXOS_OZONE_WL = "1";
    MOZ_ENABLE_WAYLAND = "1";
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
  };

  xdg.portal = {
    enable = true;
    wlr.enable = true;
    extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
    config.common.default = "*";
  };

  # ══════════════════════════════════════════════════════════════════
  # 🔧 服务与工具 (Services & Tools)
  # ══════════════════════════════════════════════════════════════════
  services.openssh.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
  };
  services.tlp.enable = true;
  programs.nix-ld.enable = true;

  hardware.graphics = {
    enable = true;
    enable32Bit = true;
    extraPackages = with pkgs; [
      intel-media-driver
      libvdpau-va-gl
    ];
  };

  virtualisation = {
    docker = {
      enable = true;
      storageDriver = "overlay2";
      autoPrune.enable = true;
    };
    libvirtd.enable = true;
  };
  programs.virt-manager.enable = true;

  programs.steam = {
    enable = true;
    remotePlay.openFirewall = true;
    gamescopeSession.enable = true;
    extraCompatPackages = with pkgs; [
      mangohud
      gamemode
    ];
  };
  programs.gamemode.enable = true;

  programs.git = {
    enable = true;
    lfs.enable = true;
    config = {
      user = {
        name = "MCB-SMART-BOY";
        email = "mcb2720838051@gmail.com";
      };
      core = {
        editor = "hx";
        pager = "delta";
      };
      interactive.diffFilter = "delta --color-only";
      delta = {
        navigate = true;
        side-by-side = true;
      };
    };
  };

  # ══════════════════════════════════════════════════════════════════
  # 📦 软件包清单 (System Packages)
  # ══════════════════════════════════════════════════════════════════
  environment.systemPackages = with pkgs; [
    # ── 网络核心 ──
    clash-verge-rev
    mihomo
    metacubexd
    # ── 终端增强 ──
    git
    wget
    curl
    eza
    bat
    ripgrep
    fd
    fzf
    zoxide
    btop
    fastfetch
    starship
    direnv
    dust
    duf
    procs
    bottom
    delta
    # ── 系统维护 ──
    gdu
    jq
    yq
    age
    sops
    lm_sensors
    usbutils
    # ── 桌面组件 ──
    wl-clipboard
    grim
    slurp
    swappy
    mako
    libnotify
    swaybg
    swaylock
    swayidle
    waybar
    fuzzel
    # ── GUI 应用 ──
    alacritty
    foot
    firefox
    google-chrome
    mpv
    vlc
    imv
    zathura
    # ── 开发环境 ──
    rustup
    gcc
    clang
    cmake
    pkg-config
    openssl
    helix
    zed-editor
    vscode-fhs
    rust-analyzer
    nil
    marksman
    taplo
    yaml-language-server
    nixfmt-rfc-style
    black
    stylua
    shfmt
    # ── 社交娱乐 ──
    qq
    telegram-desktop
    discord
    wineWowPackages.stable
    winetricks
    kazumi
    mangayomi
    bilibili
    # ── 游戏 ──
    steam
    mangohud
    protonup-qt
    lutris
    # ── 实用工具 ──
    ventoy
    qbittorrent
    aria2
    yt-dlp
    gparted
    brightnessctl
    # ── 主题美化 ──
    adwaita-icon-theme
    papirus-icon-theme
    bibata-cursors
    catppuccin-gtk
    nwg-look
    # ── 兼容层 ──
    xwayland
    xwayland-satellite
    xorg.xhost
  ];

  # ══════════════════════════════════════════════════════════════════
  # 🐚 用户与 Shell (User Config)
  # ══════════════════════════════════════════════════════════════════
  programs.appimage = {
    enable = true;
    binfmt = true;
  };

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
    linger = true; # 允许用户服务 (User Services) 驻留
  };

  # 自动创建必要的配置目录结构
  systemd.tmpfiles.rules = [
    "d /home/mcbnixos/.config/clash-verge 0750 mcbnixos users -"
    "d /var/lib/mihomo 0755 root root -"
  ];

  system.stateVersion = "25.11";
}
