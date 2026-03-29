# 用户软件声明入口（mcbnixos）。
# 这里继续作为该用户的软件总入口，但现在约定：
# 1) ./packages/ 下一个文件只对应一个软件组。
# 2) packages.nix 只负责共享上下文、导入顺序与少量本地覆盖。
# 3) 软件仍然只属于这个用户，不回流到系统层共享模块。
# 4) 小用户模板仍可保留单文件 packages.nix，不必强行拆细。

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

  # 本地自维护 Zed（来自仓库 pkgs/zed）。tryEval 失败时返回 null，避免评估报错。
  zedEval = builtins.tryEval (pkgs.callPackage ../../../pkgs/zed { });
  zedPkg = if zedEval.success then zedEval.value else null;

  # 本地自维护 YesPlayMusic（仅 x86_64-linux 可用）。
  yesplaymusicPkg =
    if pkgs.stdenv.hostPlatform.system == "x86_64-linux" then
      pkgs.callPackage ../../../pkgs/yesplaymusic { }
    else
      null;

  importPkgList =
    file:
    import file {
      inherit
        lib
        pkgs
        hostPkgEnabled
        hostNetworkGuiEnabled
        ;
    };

  proxyGui = importPkgList ./packages/proxy-gui.nix;
  bluetooth = importPkgList ./packages/bluetooth.nix;
  cliCore = importPkgList ./packages/cli-core.nix;
  shellWorkflow = importPkgList ./packages/shell-workflow.nix;
  monitors = importPkgList ./packages/monitors.nix;
  dataSecurity = importPkgList ./packages/data-security.nix;
  hardwareAndFiles = importPkgList ./packages/hardware-and-files.nix;
  waylandBase = importPkgList ./packages/wayland-base.nix;
  terminalsAndBrowsers = importPkgList ./packages/terminals-and-browsers.nix;
  mediaReaders = importPkgList ./packages/media-readers.nix;
  theming = importPkgList ./packages/theming.nix;
  xorgCompat = importPkgList ./packages/xorg-compat.nix;
  devToolchains = importPkgList ./packages/dev-toolchains.nix;
  editorsAndIde = importPkgList ./packages/editors-and-ide.nix;
  languageTools = importPkgList ./packages/language-tools.nix;
  diagrams = importPkgList ./packages/diagrams.nix;
  chatApps = importPkgList ./packages/chat-apps.nix;
  emulation = importPkgList ./packages/emulation.nix;
  officeNotes = importPkgList ./packages/office-notes.nix;
  researchWriting = importPkgList ./packages/research-writing.nix;
  creation = importPkgList ./packages/creation.nix;
  lifeTools = importPkgList ./packages/life-tools.nix;
  animeManga = importPkgList ./packages/anime-manga.nix;
  musicApps = importPkgList ./packages/music-apps.nix;
  gaming = importPkgList ./packages/gaming.nix;
  downloadAndSystem = importPkgList ./packages/download-and-system.nix;
  debugAndAnalysis = importPkgList ./packages/debug-and-analysis.nix;

  # 自维护桌面应用覆盖：
  # 1) zedPkg 成功时加入 home.packages。
  # 2) yesplaymusicPkg 在 x86_64-linux 成功时加入 home.packages。
  desktopOverrides =
    lib.optionals (zedPkg != null) [ zedPkg ]
    ++ lib.optionals (yesplaymusicPkg != null) [ yesplaymusicPkg ];
in
{
  # 让桌面入口模块按实际可用性决定是否写入 .desktop 文件。
  mcb.desktopEntries = {
    enableZed = zedPkg != null; # 仅当本地 Zed derivation 可评估时启用
    enableYesPlayMusic = yesplaymusicPkg != null; # 仅 x86_64-linux 且 derivation 可评估时启用
  };

  # 组合最终用户软件清单：
  # 1) 每个组都来自 ./packages/ 下的独立文件。
  # 2) 顺序仍然显式保留，方便阅读、局部调整和以后让 TUI 只改某一组。
  home.packages = lib.concatLists [
    proxyGui
    bluetooth
    cliCore
    shellWorkflow
    monitors
    dataSecurity
    hardwareAndFiles
    waylandBase
    terminalsAndBrowsers
    mediaReaders
    theming
    xorgCompat
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
    musicApps
    gaming
    downloadAndSystem
    debugAndAnalysis
    desktopOverrides
  ];
}
