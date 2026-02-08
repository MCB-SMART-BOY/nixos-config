# 桌面会话基础设置：niri、greetd、输入法环境变量、xdg-portal。
# 影响图形登录与 Wayland 应用的基础环境。
# 注意：输入法变量在 Home Manager 也会设置一份，保证 GUI 会话可见。

{ pkgs, lib, ... }:

{
  programs.niri.enable = true;
  programs.dconf.enable = true;
  programs.xwayland.enable = true;
  services.xserver.xkb.options = "ctrl:swapcaps";
  console.useXkbConfig = true;

  # 登录管理器：使用 greetd + tuigreet 启动 niri-session
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

  environment.sessionVariables = {
    # Wayland 优化与输入法环境变量
    NIXOS_OZONE_WL = "1";
    MOZ_ENABLE_WAYLAND = "1";
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    SDL_IM_MODULE = "fcitx";
    GLFW_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
    XIM_SERVERS = "fcitx";
    # 修复 fcitx5 插件未被发现：让 GUI 会话能找到系统共享数据目录
    XDG_DATA_DIRS = lib.mkDefault [
      "/run/current-system/sw/share"
      "/var/lib/flatpak/exports/share"
    ];
  };

  xdg.portal = {
    # Portal 让截图/文件选择等桌面能力可用
    enable = true;
    # niri 的屏幕共享默认用 GNOME portal；Screencast 单独切到 wlr（可调格式兼容性）
    wlr.enable = true;
    wlr.settings = {
      screencast = {
        # 多 GPU / DMABUF 兼容性问题时强制线性 modifier
        force_mod_linear = true;
      };
    };
    extraPortals = [
      pkgs.xdg-desktop-portal-gnome
      pkgs.xdg-desktop-portal-gtk
    ];
    config = {
      common.default = [
        "gnome"
        "gtk"
      ];
      common."org.freedesktop.impl.portal.ScreenCast" = [ "wlr" ];
      niri.default = [
        "gnome"
        "gtk"
      ];
      niri."org.freedesktop.impl.portal.ScreenCast" = [ "wlr" ];
    };
  };
}
