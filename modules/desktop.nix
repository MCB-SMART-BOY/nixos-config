# 桌面会话基础设置：niri、greetd、输入法环境变量、xdg-portal。
# 影响图形登录与 Wayland 应用的基础环境。
# 注意：输入法变量在 Home Manager 也会设置一份，保证 GUI 会话可见。

{ pkgs, ... }:

{
  programs.niri.enable = true;
  programs.dconf.enable = true;
  programs.xwayland.enable = true;

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
  };

  xdg.portal = {
    # Portal 让截图/文件选择等桌面能力可用
    enable = true;
    wlr.enable = true;
    extraPortals = [ pkgs.xdg-desktop-portal-gtk ];
    config.common.default = "*";
  };
}
