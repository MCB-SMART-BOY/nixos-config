# Home Manager 桌面应用与输入法环境变量。

{
  config,
  pkgs,
  lib,
  inputs,
  ...
}:

let
  noctaliaCfg = config.mcb.noctalia;
  scriptsRs = pkgs.callPackage ../../pkgs/scripts-rs { };

  defaultNoctaliaSettings = {
    bar = {
      widgets = {
        left = [
          { id = "Launcher"; }
          { id = "Workspace"; }
        ];
        center = [
          { id = "Clock"; }
        ];
        right = [
          { id = "Tray"; }
          { id = "Volume"; }
          { id = "Brightness"; }
          { id = "Battery"; }
          { id = "NotificationHistory"; }
          { id = "ControlCenter"; }
        ];
      };
    };
  };

  unstablePkgs = import inputs.nixpkgs-unstable {
    system = pkgs.stdenv.hostPlatform.system;
    config = pkgs.config;
  };
  legacyPkgs = import inputs.nixpkgs-24_11 {
    system = pkgs.stdenv.hostPlatform.system;
    config = pkgs.config;
  };

  xwaylandBridgePkg =
    let
      stableEval =
        if pkgs ? xwaylandvideobridge then
          builtins.tryEval pkgs.xwaylandvideobridge
        else
          {
            success = false;
            value = null;
          };
      unstableEval =
        if unstablePkgs ? xwaylandvideobridge then
          builtins.tryEval unstablePkgs.xwaylandvideobridge
        else
          {
            success = false;
            value = null;
          };
      legacyEval =
        if legacyPkgs ? xwaylandvideobridge then
          builtins.tryEval legacyPkgs.xwaylandvideobridge
        else
          {
            success = false;
            value = null;
          };
    in
    if stableEval.success then
      stableEval.value
    else if unstableEval.success then
      unstableEval.value
    else if legacyEval.success then
      legacyEval.value
    else
      null;

in
{
  options.mcb = {
    desktopEntries = {
      enableZed = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Enable Zed desktop entry override for this user.";
      };

      enableYesPlayMusic = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Enable YesPlayMusic desktop entry override for this user.";
      };
    };

    noctalia = {
      barProfile = lib.mkOption {
        type = lib.types.enum [
          "default"
          "none"
        ];
        default = "default";
        description = "Noctalia bar profile: default (built-in widgets) or none (disable managed bar settings).";
      };
    };
  };

  imports = [
    inputs.noctalia.homeModules.default
  ];

  config = {
  # 使用 Noctalia 作为桌面 Shell
  programs.noctalia-shell.enable = true;
  programs.noctalia-shell.settings =
    if noctaliaCfg.barProfile == "default" then
      defaultNoctaliaSettings
    else
      { };

  home.sessionVariables = {
    # 输入法环境变量（保证 Wayland 应用能读取）
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    SDL_IM_MODULE = "fcitx";
    GLFW_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
    XIM_SERVERS = "fcitx";
  };

  home.packages = lib.optionals (xwaylandBridgePkg != null) [ xwaylandBridgePkg ] ++ [
    scriptsRs
  ];

  # 论文相关文件默认使用 LibreOffice / Sioyek，避免 WPS 抢占默认关联。
  xdg.mimeApps = {
    enable = true;
    defaultApplications = {
      # PDF / PostScript
      "application/pdf" = [ "sioyek.desktop" ];
      "application/postscript" = [ "sioyek.desktop" ];
      # Word 文档
      "application/msword" = [ "libreoffice-writer.desktop" ];
      "application/rtf" = [ "libreoffice-writer.desktop" ];
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document" = [
        "libreoffice-writer.desktop"
      ];
      "application/vnd.oasis.opendocument.text" = [ "libreoffice-writer.desktop" ];
      # Excel 表格
      "application/vnd.ms-excel" = [ "libreoffice-calc.desktop" ];
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" = [
        "libreoffice-calc.desktop"
      ];
      "application/vnd.oasis.opendocument.spreadsheet" = [ "libreoffice-calc.desktop" ];
      # PowerPoint 演示
      "application/vnd.ms-powerpoint" = [ "libreoffice-impress.desktop" ];
      "application/vnd.openxmlformats-officedocument.presentationml.presentation" = [
        "libreoffice-impress.desktop"
      ];
      "application/vnd.oasis.opendocument.presentation" = [ "libreoffice-impress.desktop" ];
      # 文献管理
      "x-scheme-handler/zotero" = [ "zotero.desktop" ];
      "text/x-bibtex" = [ "zotero.desktop" ];
      "application/x-research-info-systems" = [ "zotero.desktop" ];
    };
  };

  xdg.desktopEntries."sioyek" = {
    name = "Sioyek";
    genericName = "PDF Viewer";
    comment = "PDF viewer optimized for research papers";
    exec = "sioyek %U";
    icon = "sioyek";
    categories = [
      "Office"
      "Viewer"
    ];
    mimeType = [
      "application/pdf"
      "application/postscript"
    ];
    startupNotify = true;
    terminal = false;
  };

  xdg.desktopEntries."zotero" = {
    name = "Zotero";
    genericName = "Reference Manager";
    comment = "Collect, organize and cite research";
    exec = "zotero %U";
    icon = "zotero";
    categories = [
      "Office"
      "Education"
      "Science"
    ];
    mimeType = [
      "x-scheme-handler/zotero"
      "text/x-bibtex"
      "application/x-research-info-systems"
    ];
    startupNotify = true;
    terminal = false;
  };

  # Override upstream desktop entry so GUI launcher also goes through adaptive wrapper.
  xdg.desktopEntries."dev.zed.Zed" = lib.mkIf config.mcb.desktopEntries.enableZed {
    name = "Zed";
    genericName = "Text Editor";
    comment = "A high-performance, multiplayer code editor.";
    exec = "zed-auto-gpu %U";
    icon = "zed";
    categories = [
      "Utility"
      "TextEditor"
      "Development"
      "IDE"
    ];
    mimeType = [
      "text/plain"
      "application/x-zerosize"
      "x-scheme-handler/zed"
    ];
    startupNotify = true;
    terminal = false;
  };

  xdg.desktopEntries."io.github.msojocs.bilibili" = {
    name = "Bilibili";
    comment = "Bilibili Desktop";
    exec = "electron-auto-gpu bilibili %U";
    icon = "io.github.msojocs.bilibili";
    categories = [
      "AudioVideo"
      "Video"
      "TV"
    ];
    startupNotify = true;
    terminal = false;
  };

  xdg.desktopEntries."discord" = {
    name = "Discord";
    genericName = "All-in-one cross-platform voice and text chat for gamers";
    exec = "electron-auto-gpu Discord %U";
    icon = "discord";
    categories = [
      "Network"
      "InstantMessaging"
    ];
    mimeType = [ "x-scheme-handler/discord" ];
    startupNotify = true;
    terminal = false;
  };

  xdg.desktopEntries."obsidian" = {
    name = "Obsidian";
    comment = "Knowledge base";
    exec = "electron-auto-gpu obsidian %U";
    icon = "obsidian";
    categories = [ "Office" ];
    mimeType = [ "x-scheme-handler/obsidian" ];
    startupNotify = true;
    terminal = false;
  };

  xdg.desktopEntries."clash-verge" = {
    name = "Clash Verge";
    comment = "Clash Verge Rev";
    exec = "electron-auto-gpu clash-verge %U";
    icon = "clash-verge";
    categories = [ "Development" ];
    mimeType = [ "x-scheme-handler/clash" ];
    startupNotify = true;
    terminal = false;
  };

  xdg.desktopEntries."clash-nyanpasu" = {
    name = "Clash Nyanpasu";
    comment = "Clash Nyanpasu! (∠・ω< )⌒☆";
    exec = "electron-auto-gpu clash-nyanpasu";
    icon = "clash-nyanpasu";
    categories = [ "Development" ];
    startupNotify = true;
    terminal = false;
  };

  xdg.desktopEntries."yesplaymusic" = lib.mkIf config.mcb.desktopEntries.enableYesPlayMusic {
    name = "YesPlayMusic";
    comment = "A third-party music player for Netease Music";
    exec = "electron-auto-gpu yesplaymusic --no-sandbox %U";
    icon = "yesplaymusic";
    categories = [
      "AudioVideo"
      "Audio"
      "Player"
      "Music"
    ];
    startupNotify = true;
    terminal = false;
  };

  systemd.user.services.xwaylandvideobridge = lib.mkIf (xwaylandBridgePkg != null) {
    Unit = {
      Description = "XWayland Video Bridge (screen sharing for X11 apps)";
      After = [
        "graphical-session.target"
        "pipewire.service"
        "xdg-desktop-portal.service"
      ];
      PartOf = [ "graphical-session.target" ];
      Wants = [
        "pipewire.service"
        "xdg-desktop-portal.service"
      ];
      ConditionPathExistsGlob = "%t/wayland-*";
    };
    Service = {
      ExecStart = "${xwaylandBridgePkg}/bin/xwaylandvideobridge";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
  };
}
