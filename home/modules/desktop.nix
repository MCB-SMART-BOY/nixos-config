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

  # Zed launcher that adapts backend/GPU selection by current specialisation mode.
  # dgpu mode uses OpenGL backend to avoid known niri+NVIDIA Wayland stale-frame issues.
  zedAutoGpu = pkgs.writeShellApplication {
    name = "zed-auto-gpu";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.gnused
    ];
    text = ''
      set -euo pipefail

      mode_from_path() {
        local path="$1"
        local mode=""

        if [[ "$path" == */specialisation/gpu-* ]]; then
          mode="''${path##*/specialisation/}"
          mode="''${mode%%/*}"
        elif [[ "$path" == *specialisation-gpu-* ]]; then
          mode="''${path##*specialisation-}"
          mode="''${mode%%/*}"
        fi

        if [[ -n "$mode" ]]; then
          printf '%s' "$mode"
          return 0
        fi
        return 1
      }

      current_mode() {
        local path=""
        local mode=""
        local token=""
        local cmd_path=""

        path="$(readlink -f /run/current-system 2>/dev/null || true)"
        mode="$(mode_from_path "$path" 2>/dev/null || true)"
        if [[ -n "$mode" ]]; then
          printf '%s' "$mode"
          return 0
        fi

        path="$(readlink -f /run/booted-system 2>/dev/null || true)"
        mode="$(mode_from_path "$path" 2>/dev/null || true)"
        if [[ -n "$mode" ]]; then
          printf '%s' "$mode"
          return 0
        fi

        if [[ -r /proc/cmdline ]]; then
          for token in $(</proc/cmdline); do
            case "$token" in
              init=*|systemConfig=*)
                cmd_path="''${token#*=}"
                ;;
              *)
                cmd_path=""
                ;;
            esac

            if [[ -n "$cmd_path" ]]; then
              mode="$(mode_from_path "$cmd_path" 2>/dev/null || true)"
              if [[ -n "$mode" ]]; then
                printf '%s' "$mode"
                return 0
              fi
            fi
          done
        fi

        if command -v noctalia-gpu-mode >/dev/null 2>&1; then
          mode="$(noctalia-gpu-mode 2>/dev/null | sed -n 's/.*specialisation: \([^"]*\).*/\1/p' | head -n 1 || true)"
          if [[ -n "$mode" ]]; then
            printf '%s' "$mode"
            return 0
          fi
        fi

        printf '%s' "base"
      }

      mode="$(current_mode)"
      case "$mode" in
        gpu-dgpu|dgpu)
          # Keep hardware acceleration on dGPU while avoiding stale-frame bug on Wayland.
          export WGPU_BACKEND="''${WGPU_BACKEND:-gl}"
          export __GLX_VENDOR_LIBRARY_NAME="nvidia"
          export __VK_LAYER_NV_optimus="NVIDIA_only"
          ;;
        *)
          # iGPU/hybrid/base: avoid carrying stale offload env into Zed.
          unset __NV_PRIME_RENDER_OFFLOAD
          unset __NV_PRIME_RENDER_OFFLOAD_PROVIDER
          unset __GLX_VENDOR_LIBRARY_NAME
          unset __VK_LAYER_NV_optimus
          unset DRI_PRIME
          unset WGPU_BACKEND
          ;;
      esac

      exec zeditor "$@"
    '';
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

  home.packages =
    lib.optionals (xwaylandBridgePkg != null) [ xwaylandBridgePkg ]
    ++ [ zedAutoGpu ];

  # Override upstream desktop entry so GUI launcher also goes through adaptive wrapper.
  xdg.desktopEntries."dev.zed.Zed" = {
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
