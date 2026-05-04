# 用户软件声明入口（mcbnixos）：逐个写包并附用途说明。
# 说明：
# 1) 每个分组都可以按需增删，不影响其他用户。
# 2) 这些包进入当前用户的 Home Manager profile（home.packages）。
# 3) 对于已由 mcb.packages.* 提供的系统级共享包，会在这里自动去重（避免重复声明）。
# 4) 注释尽量写到包级别，方便后续快速维护。

{
  lib,
  pkgs,
  osConfig ? { },
  ...
}:

let
  hostPkgCfg = lib.attrByPath [ "mcb" "packages" ] { } osConfig;
  hostPkgEnabled = name: lib.attrByPath [ name ] false hostPkgCfg;
  hostNetworkGuiEnabled = (hostPkgEnabled "enableNetwork") || (hostPkgEnabled "enableNetworkGui");

  # 本地自维护 YesPlayMusic（仅 x86_64-linux 可用）。
  yesplaymusicPkg =
    if pkgs.stdenv.hostPlatform.system == "x86_64-linux" then
      pkgs.callPackage ../../../pkgs/yesplaymusic { }
    else
      null;

  # 网络 / 代理可视化前端（桌面图形工具）。
  proxyGui = with pkgs; [
    clash-nyanpasu # Clash GUI（MetaCubeX 生态）
    metacubexd # MetaCubeX 仪表盘/控制前端
  ];

  # 蓝牙用户态工具（托盘 + 管理界面 + 命令行）。
  bluetooth = with pkgs; [
    bluez # 蓝牙协议栈核心工具集
    bluez-tools # 蓝牙命令行辅助工具
    blueman # 蓝牙图形管理器
  ];

  # CLI 核心工具（拉代码、下载、查找、查看）。
  cliCore = with pkgs; [
    git # 版本控制
    lazygit # Git TUI
    wget # 文件下载（非交互）
    curl # HTTP 调试/下载
    eza # ls 增强
    fd # find 替代，速度快
    fzf # 模糊搜索
    ripgrep # 全文搜索
    bat # cat 高亮版
    delta # git diff 美化
  ];

  # Shell 工作流增强（提示、自动补全、目录跳转）。
  shellWorkflow = with pkgs; [
    zoxide # 智能目录跳转
    starship # Shell Prompt
    direnv # 目录级环境变量管理
    oh-my-zsh # Zsh 插件框架
    zsh-autosuggestions # Zsh 自动建议
    zsh-syntax-highlighting # Zsh 语法高亮
    zsh-completions # 补全扩展
    fish # 另一套交互式 shell
  ];

  # 系统状态监控（CPU/内存/磁盘/进程）。
  monitors = with pkgs; [
    btop # 现代资源监控 TUI
    bottom # 资源监控 TUI（另一个风格）
    fastfetch # 系统信息展示
    duf # 磁盘占用（df 替代）
    gdu # 磁盘空间分析
    dust # 目录体积分析
    procs # ps 增强版
  ];

  # 数据处理与密钥/密文工具。
  dataSecurity = with pkgs; [
    jq # JSON 处理
    yq # YAML/JSON 处理
    age # 现代加密工具
    sops # 密文配置管理
  ];

  # 硬件信息与终端文件管理。
  hardwareAndFiles = with pkgs; [
    lm_sensors # 传感器信息（温度/风扇）
    usbutils # USB 设备信息（lsusb）
    yazi # 终端文件管理器
  ];

  # Wayland 桌面基础（剪贴板/截图/通知/会话控制）。
  waylandBase = with pkgs; [
    wl-clipboard # Wayland 剪贴板
    grim # Wayland 截图
    slurp # 区域选择（截图配套）
    swappy # 截图标注
    libnotify # 桌面通知接口
    fuzzel # Wayland launcher
    swayidle # 空闲管理
    niri # Wayland compositor（会话组件）
    pipewire # 多媒体管线
    brightnessctl # 背光控制
  ];

  # 终端与浏览器。
  terminalsAndBrowsers = with pkgs; [
    foot # 轻量 Wayland 终端
    alacritty # GPU 加速终端
    firefox # 浏览器（开源）
    google-chrome # 浏览器（Chromium 系）
  ];

  # 媒体与阅读应用。
  mediaReaders = with pkgs; [
    nautilus # 文件管理器（GUI）
    mpv # 视频播放器
    vlc # 多媒体播放器
    imv # 图片查看器（Wayland 友好）
    zathura # PDF 阅读器（键盘友好）
  ];

  # 开发工具链（编译器/构建器/语言安装器）。
  devToolchains = with pkgs; [
    rustup # Rust 工具链管理
    opam # OCaml 包管理
    elan # Lean 工具链管理
    gnumake # make 构建
    cmake # C/C++ 构建系统
    pkg-config # 库探测
    openssl # TLS 库与工具
    gcc # GNU C/C++ 编译器
    binutils # 链接器/二进制工具
    clang-tools # Clang 工具集
    uv # Python 包与虚拟环境工具
    conda # Python/数据科学环境管理
  ];

  # 编辑器 / IDE 主程序。
  editorsAndIde = with pkgs; [
    neovim # 模态编辑器
    helix # 模态编辑器（现代配置）
    vscode-fhs # VS Code（FHS 兼容封装）
    zed-editor-fhs
  ];

  # 语言服务器与格式化工具（LSP / Formatter）。
  languageTools = with pkgs; [
    typescript-language-server # TS/JS LSP
    prettier # 通用格式化器
    bash-language-server # Bash LSP
    pyright # Python 类型检查/LSP
    vscode-langservers-extracted # HTML/CSS/JSON/ESLint 等 LSP
    nixd # Nix LSP
    marksman # Markdown LSP
    taplo # TOML 工具/LSP
    yaml-language-server # YAML LSP
    lua-language-server # Lua LSP
    gopls # Go LSP
    nixfmt # Nix 格式化
    black # Python 格式化
    stylua # Lua 格式化
    shfmt # Shell 格式化
  ];

  # 图表 / UML / 流程图。
  diagrams = with pkgs; [
    drawio # 绘图与流程图工具
  ];

  # 通讯社交客户端。
  chatApps = with pkgs; [
    qq # QQ 客户端
    telegram-desktop # Telegram 客户端
    discord # Discord 客户端
  ];

  # Windows 兼容层。
  emulation = with pkgs; [
    wineWowPackages.stable # Wine（32/64 位兼容）
    winetricks # Wine 运行库安装脚本
  ];

  # 办公与知识管理。
  officeNotes = with pkgs; [
    obsidian # Markdown 知识库
    libreoffice-still # 办公套件（稳定分支）
    xournalpp # 手写笔记/PDF 标注
    goldendict-ng # 词典工具
  ];

  # 写作科研工具链（文献/排版/论文）。
  researchWriting = with pkgs; [
    sioyek # 学术 PDF 阅读器
    zotero # 文献管理
    pandoc # 文档格式转换
    typst # 新一代排版引擎
    tinymist # Typst LSP
    texstudio # LaTeX IDE
    texlab # LaTeX LSP
    texlive.combined.scheme-medium # TeX Live 中型套装
    biber # BibLaTeX 参考文献工具
    qpdf # PDF 结构处理
    poppler-utils # PDF 命令行工具（pdftotext 等）
  ];

  # 内容创作（录屏/直播/混流）。
  creation = with pkgs; [
    obs-studio # 录屏与直播
  ];

  # 日常生活工具。
  lifeTools = with pkgs; [
    gnome-calendar # 日历
    gnome-clocks # 时钟
    gnome-calculator # 计算器
    gnome-weather # 天气
    gnome-maps # 地图
    gnome-contacts # 通讯录
    baobab # 磁盘占用可视化
    keepassxc # 密码管理
    simple-scan # 扫描仪前端
  ];

  # 动漫 / 漫画 / 二次元视频工具。
  animeManga =
    (with pkgs; [
      kazumi # 动漫聚合客户端
      mangayomi # 漫画/视频聚合
      bilibili # 哔哩哔哩客户端
      ani-cli # 终端动漫工具
      mangal # 漫画下载/阅读 CLI
      venera # 漫画下载/阅读 GUI
    ]);

  # 音乐播放相关。
  musicApps = with pkgs; [
    ncspot # Spotify TUI 客户端
    mpd # 音乐守护进程
    ncmpcpp # MPD TUI 客户端
    playerctl # 媒体键控制
  ];

  # 游戏运行与平台工具。
  gaming = with pkgs; [
    mangohud # 游戏性能叠层
    protonup-qt # Proton-GE 管理
    lutris # 游戏启动器/整合器
  ];

  # 下载与系统控制面板。
  downloadAndSystem = with pkgs; [
    qbittorrent # BT 下载
    aria2 # 多协议下载
    yt-dlp # 视频下载
    gparted # 分区管理
    pavucontrol # 音频设备控制
  ];

  # 主题、图标、光标、GTK 外观。
  theming = with pkgs; [
    adwaita-icon-theme # GNOME 默认图标
    gnome-themes-extra # GNOME 主题补充
    papirus-icon-theme # Papirus 图标主题
    bibata-cursors # 鼠标光标主题
    catppuccin-gtk # GTK 主题
    nwg-look # GTK/图标主题切换 GUI
  ];

  # Xwayland 兼容层工具（混合运行 X11 程序时常用）。
  xorgCompat = with pkgs; [
    xwayland # Xwayland 服务
    xwayland-satellite # Xwayland 集成辅助
    xorg.xhost # X11 访问控制工具
  ];

  # 调试、分析、抓包、性能测试与二进制工具。
  debugAndAnalysis = with pkgs; [
    strace # 系统调用跟踪
    ltrace # 动态库调用跟踪
    gdb # GNU 调试器
    lldb # LLVM 调试器
    patchelf # ELF 修改工具
    file # 文件类型识别
    htop # 进程查看器
    iotop # IO 占用监控
    iftop # 网络流量监控
    sysstat # 系统性能统计（sar/iostat）
    lsof # 文件句柄/端口占用查询
    mtr # 路由诊断
    nmap # 端口扫描
    tcpdump # 抓包 CLI
    traceroute # 路由追踪
    socat # 套接字转发/桥接
    iperf3 # 网络带宽测试
    ethtool # 网卡参数查询/调优
    hyperfine # 命令基准测试
    tokei # 代码行数统计
    tree # 目录树查看
    unzip # 解压 zip
    zip # 打包 zip
    p7zip # 7z 压缩工具
    rsync # 增量同步
    rclone # 多云存储同步
    just # 任务运行器
    entr # 文件变化触发命令
    ncdu # 磁盘占用 TUI
    binwalk # 固件分析
    radare2 # 逆向分析框架
    wireshark # 图形化抓包工具
    vulkan-tools # Vulkan 诊断工具
    gh # GitHub CLI
    hexyl # 十六进制查看器
  ];

  # 自维护桌面应用覆盖
  desktopOverrides = lib.optionals (yesplaymusicPkg != null) [ yesplaymusicPkg ];
in
{
  # 让桌面入口模块按实际可用性决定是否写入 .desktop 文件。
  mcb.desktopEntries = {
    enableYesPlayMusic = yesplaymusicPkg != null; # 仅 x86_64-linux 且 derivation 可评估时启用
  };

  # 组合最终用户软件清单。分组顺序即安装声明顺序，方便阅读与维护。
  home.packages = lib.concatLists [
    (lib.optionals (!hostNetworkGuiEnabled) proxyGui)
    (lib.optionals (!hostNetworkGuiEnabled) bluetooth)
    (lib.optionals (!(hostPkgEnabled "enableShellTools")) cliCore)
    (lib.optionals (!(hostPkgEnabled "enableShellTools")) shellWorkflow)
    (lib.optionals (!(hostPkgEnabled "enableShellTools")) monitors)
    (lib.optionals (!(hostPkgEnabled "enableShellTools")) dataSecurity)
    (lib.optionals (!(hostPkgEnabled "enableShellTools")) hardwareAndFiles)
    (lib.optionals (!(hostPkgEnabled "enableWaylandTools")) waylandBase)
    terminalsAndBrowsers
    mediaReaders
    devToolchains
    editorsAndIde
    languageTools
    diagrams
    chatApps
    emulation
    officeNotes
    researchWriting
    creation
    lifeTools
    animeManga
    (lib.optionals (!(hostPkgEnabled "enableMusic")) musicApps)
    (lib.optionals (!(hostPkgEnabled "enableGaming")) gaming)
    (lib.optionals (!(hostPkgEnabled "enableSystemTools")) downloadAndSystem)
    (lib.optionals (!(hostPkgEnabled "enableTheming")) theming)
    (lib.optionals (!(hostPkgEnabled "enableXorgCompat")) xorgCompat)
    (lib.optionals (!(hostPkgEnabled "enableGeekTools")) debugAndAnalysis)
    desktopOverrides
  ];
}
