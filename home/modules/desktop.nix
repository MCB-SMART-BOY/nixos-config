# Home Manager 桌面应用与输入法环境变量。

{
  pkgs,
  lib,
  inputs,
  ...
}:

let
  xwaylandBridgeEval =
    if pkgs ? xwaylandvideobridge then
      builtins.tryEval pkgs.xwaylandvideobridge
    else
      {
        success = false;
        value = null;
      };
in
{
  imports = [
    inputs.noctalia.homeModules.default
  ];

  # 使用 Noctalia 作为桌面 Shell
  programs.noctalia-shell.enable = true;

  home.sessionVariables = {
    # 输入法环境变量（保证 Wayland 应用能读取）
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    SDL_IM_MODULE = "fcitx";
    GLFW_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
    XIM_SERVERS = "fcitx";
  };

  systemd.user.services.xwaylandvideobridge = lib.mkIf xwaylandBridgeEval.success {
    Unit = {
      Description = "XWayland Video Bridge (screen sharing for X11 apps)";
      After = [ "graphical-session.target" ];
      PartOf = [ "graphical-session.target" ];
      ConditionPathExistsGlob = "%t/wayland-*";
    };
    Service = {
      ExecStart = "${xwaylandBridgeEval.value}/bin/xwaylandvideobridge";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
