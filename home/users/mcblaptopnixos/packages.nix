# 用户软件模板（mcblaptopnixos）：分组清晰，但保持中等复杂度。

{ lib, pkgs, ... }:

let
  # 自维护桌面应用（可选）
  zedPkg = pkgs.zed-editor;
  yesplaymusicPkg =
    if pkgs.stdenv.hostPlatform.system == "x86_64-linux" then
      pkgs.callPackage ../../../pkgs/yesplaymusic { }
    else
      null;

  # 桌面基础（终端、浏览器、文件与媒体）
  desktopBase = with pkgs; [
    foot
    alacritty
    firefox
    google-chrome
    nautilus
    mpv
    vlc
    imv
    zathura
  ];

  # 开发工具（工具链 + 编辑器）
  devBase = with pkgs; [
    rustup
    opam
    elan
    gnumake
    cmake
    pkg-config
    openssl
    gcc
    binutils
    clang-tools
    uv
    conda
    neovim
    helix
    vscode-fhs
    isabelle
    drawio
  ];

  # LSP / 格式化
  devLanguageTools = with pkgs; [
    nodePackages.typescript-language-server
    nodePackages.prettier
    bash-language-server
    pyright
    vscode-langservers-extracted
    nixd
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

  # 社交与兼容层
  socialAndCompatibility = with pkgs; [
    qq
    telegram-desktop
    discord
    wineWowPackages.stable
    winetricks
  ];

  # 办公与学习
  officeAndStudy = with pkgs; [
    obsidian
    obs-studio
    libreoffice-still
    xournalpp
    sioyek
    zotero
    pandoc
    typst
    tinymist
    texstudio
    texlab
    texlive.combined.scheme-medium
    biber
    qpdf
    poppler-utils
    goldendict-ng
  ];

  # 日常工具与娱乐
  lifeAndEntertainment =
    (with pkgs; [
      gnome-calendar
      gnome-clocks
      gnome-calculator
      gnome-weather
      gnome-maps
      gnome-contacts
      baobab
      keepassxc
      simple-scan
      kazumi
      mangayomi
      bilibili
      ani-cli
      mangal
    ])
    ++ lib.optionals (pkgs ? venera) [ pkgs.venera ];

  # 自维护应用覆盖（按平台和构建可用性自动决定）
  desktopOverrides =
    lib.optionals (zedPkg != null) [ zedPkg ]
    ++ lib.optionals (yesplaymusicPkg != null) [ yesplaymusicPkg ];
in
{
  mcb.desktopEntries = {
    enableZed = zedPkg != null;
    enableYesPlayMusic = yesplaymusicPkg != null;
  };

  home.packages = lib.concatLists [
    desktopBase
    devBase
    devLanguageTools
    socialAndCompatibility
    officeAndStudy
    lifeAndEntertainment
    desktopOverrides
  ];
}
