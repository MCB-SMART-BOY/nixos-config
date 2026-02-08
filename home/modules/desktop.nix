# Home Manager 桌面应用与输入法环境变量。

{
  pkgs,
  lib,
  inputs,
  ...
}:

let
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

  home.packages = lib.optionals (xwaylandBridgePkg != null) [ xwaylandBridgePkg ];

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
}
