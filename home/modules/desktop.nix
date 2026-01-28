# Home Manager 桌面应用与输入法环境变量。

{ ... }:

{
  # 仅开启 Home Manager 层的桌面组件开关
  programs.swaylock.enable = true;
  programs.fuzzel.enable = true;
  programs.waybar.enable = true;

  home.sessionVariables = {
    # 输入法环境变量（保证 Wayland 应用能读取）
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    SDL_IM_MODULE = "fcitx";
    GLFW_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
    XIM_SERVERS = "fcitx";
  };
}
