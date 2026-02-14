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

  # Normalize current GPU mode for wrapper scripts (igpu/hybrid/dgpu/base).
  gpuModeCurrent = pkgs.writeShellApplication {
    name = "noctalia-gpu-current";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.gnused
    ];
    text = ''
      set -euo pipefail

      normalize_mode() {
        case "$1" in
          gpu-dgpu|dgpu)
            printf '%s\n' "dgpu"
            ;;
          gpu-hybrid|hybrid)
            printf '%s\n' "hybrid"
            ;;
          gpu-igpu|igpu)
            printf '%s\n' "igpu"
            ;;
          *)
            printf '%s\n' "base"
            ;;
        esac
      }

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
          normalize_mode "$mode"
          return 0
        fi
        return 1
      }

      if [[ -n "''${MCB_GPU_MODE-}" ]]; then
        normalize_mode "''${MCB_GPU_MODE}"
        exit 0
      fi

      path="$(readlink -f /run/current-system 2>/dev/null || true)"
      mode="$(mode_from_path "$path" 2>/dev/null || true)"
      if [[ -n "$mode" ]]; then
        printf '%s\n' "$mode"
        exit 0
      fi

      path="$(readlink -f /run/booted-system 2>/dev/null || true)"
      mode="$(mode_from_path "$path" 2>/dev/null || true)"
      if [[ -n "$mode" ]]; then
        printf '%s\n' "$mode"
        exit 0
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
              printf '%s\n' "$mode"
              exit 0
            fi
          fi
        done
      fi

      if command -v noctalia-gpu-mode >/dev/null 2>&1; then
        mode="$(noctalia-gpu-mode 2>/dev/null | sed -n 's/.*specialisation: \([^"]*\).*/\1/p' | head -n 1 || true)"
        if [[ -n "$mode" ]]; then
          normalize_mode "$mode"
          exit 0
        fi
      fi

      printf '%s\n' "base"
    '';
  };

  # Zed launcher that adapts backend/GPU selection by current specialisation mode.
  # dgpu mode uses OpenGL backend to avoid known niri+NVIDIA Wayland stale-frame issues.
  zedAutoGpu = pkgs.writeShellApplication {
    name = "zed-auto-gpu";
    runtimeInputs = [ gpuModeCurrent ];
    text = ''
      set -euo pipefail

      mode="$(noctalia-gpu-current 2>/dev/null || printf '%s' base)"
      case "$mode" in
        dgpu)
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

  # Generic Electron wrapper: force stable X11 rendering path on dGPU mode.
  electronAutoGpu = pkgs.writeShellApplication {
    name = "electron-auto-gpu";
    runtimeInputs = [ gpuModeCurrent ];
    text = ''
      set -euo pipefail

      if [[ $# -lt 1 ]]; then
        echo "Usage: electron-auto-gpu <command> [args...]" >&2
        exit 2
      fi

      app="$1"
      shift

      if ! command -v "$app" >/dev/null 2>&1; then
        echo "electron-auto-gpu: command not found: $app" >&2
        exit 127
      fi

      mode="$(noctalia-gpu-current 2>/dev/null || printf '%s' base)"
      if [[ "$mode" == "dgpu" ]]; then
        export NIXOS_OZONE_WL="0"
        export ELECTRON_OZONE_PLATFORM_HINT="x11"
        export OZONE_PLATFORM="x11"
      fi

      exec "$app" "$@"
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

  home.packages = lib.optionals (xwaylandBridgePkg != null) [ xwaylandBridgePkg ] ++ [
    gpuModeCurrent
    zedAutoGpu
    electronAutoGpu
  ];

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

  xdg.desktopEntries."yesplaymusic" = {
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
}
