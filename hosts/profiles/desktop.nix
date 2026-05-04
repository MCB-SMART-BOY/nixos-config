# 桌面 profile：启用桌面模块与常用包组。

{ ... }:

{
  mcb.hostRole = "desktop";
  # 桌面主机默认保留用户 linger（方便 user-level 服务在注销后继续运行）
  mcb.userLinger = true;

  # 桌面主机引入完整的系统模块
  imports = [
    ./base.nix
    ../../modules/i18n.nix
    ../../modules/fonts.nix
    ../../modules/desktop.nix
    ../../modules/services/desktop.nix
    ../../modules/virtualization.nix
    ../../modules/gaming.nix
  ];

  mcb.packages = {
    # 系统级共享包组（用户个性化应用请写到 home/users/<user>/packages.nix）
    enableNetworkCli = true;
    enableNetworkGui = true;
    enableShellTools = true;
    enableWaylandTools = true;
    enableGaming = true;
    enableSystemTools = true;
    enableTheming = true;
    enableXorgCompat = true;
    enableGeekTools = true;
    enableMusic = true;
  };

  mcb.flatpak = {
    enable = true;
    apps = [
      "com.tencent.WeChat"
      "com.tencent.wemeet"
    ];
    overrides = {
      # 腾讯系应用需要 X11 回退（Wayland 兼容性差）
      # GPU 渲染（--device=dri）+ IPC + 输入法
      filesystem = [
        "xdg-desktop"
        "xdg-documents"
        "xdg-download"
        "xdg-music"
        "xdg-pictures"
        "xdg-public-share"
        "xdg-videos"
        "home"
      ];
      env = {
        "QT_QPA_PLATFORM" = "xcb";
        "QT_IM_MODULE" = "fcitx";
        "GTK_IM_MODULE" = "fcitx";
        "XMODIFIERS" = "@im=fcitx";
      };
      extraArgs = [
        "--device=dri"
        "--socket=x11"
        "--socket=wayland"
        "--socket=pulseaudio"
        "--share=ipc"
      ];
    };
  };
}
