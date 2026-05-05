# 桌面服务：音频、图形驱动、AppImage、节能等。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  flatpakCfg = config.mcb.flatpak;

  # 腾讯会议 libImSDK.so 在多线程中直接调用 X11 但不调用 XInitThreads()
  # → XSetInputFocus Display hook 指针并发损坏 → SIGSEGV。
  # 修复: LD_PRELOAD interpose XOpenDisplay，首次调用前自动调 XInitThreads()
  wemeetX11ThreadFix = pkgs.stdenv.mkDerivation {
    name = "wemeet-x11-thread-fix";
    dontUnpack = true;
    buildPhase = ''
      $CC -shared -fPIC -o libx11threadfix.so -xc -ldl - <<'SRC'
        #define _GNU_SOURCE
        #include <dlfcn.h>
        typedef void* (*XOD_type)(const char*);
        typedef int   (*XIT_type)(void);
        static int done = 0;
        void* XOpenDisplay(const char *name) {
          if (!done) {
            XIT_type f = dlsym(RTLD_NEXT, "XInitThreads");
            if (f) f();
            done = 1;
          }
          XOD_type real = dlsym(RTLD_NEXT, "XOpenDisplay");
          return real(name);
        }
      SRC
    '';
    installPhase = ''
      mkdir -p $out/lib
      cp libx11threadfix.so $out/lib/
    '';
  };

  flatpakSetupHash = builtins.substring 0 16 (
    builtins.hashString "sha256" (
      builtins.toJSON {
        enableFlathub = flatpakCfg.enableFlathub;
        apps = flatpakCfg.apps;
        overrides = flatpakCfg.overrides;
      }
    )
  );
  flatpakSetupStamp = "/var/lib/nixos-config/flatpak-setup-${flatpakSetupHash}.done";
  flatpakManagedApps = "/var/lib/nixos-config/flatpak-managed-apps";
  flatpakDesiredApps = lib.concatStringsSep " " (map lib.escapeShellArg flatpakCfg.apps);
  flatpakInstallScript = ''
    desired_apps=(${flatpakDesiredApps})
    managed_apps_file=${lib.escapeShellArg flatpakManagedApps}

    if [ -f "$managed_apps_file" ]; then
      while IFS= read -r app || [ -n "$app" ]; do
        [ -n "$app" ] || continue
        keep=false
        for desired in "''${desired_apps[@]}"; do
          if [ "$app" = "$desired" ]; then
            keep=true
            break
          fi
        done
        if [ "$keep" != true ]; then
          ${pkgs.flatpak}/bin/flatpak uninstall --system -y --noninteractive "$app" || true
        fi
      done < "$managed_apps_file"
    fi

    : > "$managed_apps_file"
    for app in "''${desired_apps[@]}"; do
      [ -n "$app" ] || continue
      ${pkgs.flatpak}/bin/flatpak install --system -y --noninteractive flathub "$app"
      ${pkgs.coreutils}/bin/printf '%s\n' "$app" >> "$managed_apps_file"
    done
  '';
  flatpakOverrideArgs =
    let
      fsArgs = map (path: "--filesystem=${lib.escapeShellArg path}") flatpakCfg.overrides.filesystem;
      envArgs = lib.mapAttrsToList (
        key: value: "--env=${lib.escapeShellArg "${key}=${value}"}"
      ) flatpakCfg.overrides.env;
      extraArgs = map lib.escapeShellArg flatpakCfg.overrides.extraArgs;
    in
    lib.concatStringsSep " " (fsArgs ++ envArgs ++ extraArgs);
  flatpakOverrideScript = lib.optionalString (flatpakOverrideArgs != "") ''
    ${pkgs.flatpak}/bin/flatpak override --system ${flatpakOverrideArgs}
  '';

  # wemeet 修复脚本
  wemeetFixScript = pkgs.writeShellScript "wemeet-fix" ''
    set -euo pipefail
    SCRIPT="/var/lib/flatpak/app/com.tencent.wemeet/current/active/files/extra/opt/wemeet/wemeetapp.sh"
    FLATPAK="${pkgs.flatpak}/bin/flatpak"
    FIX_DST="/home/mcbnixos/xinitthreads_fix.so"

    if [ -f "$SCRIPT" ]; then
      ${pkgs.gnused}/bin/sed -i '1s/^!/#!/' "$SCRIPT"

      if grep -q 'export WEMEET_XWAYLAND=1' "$SCRIPT" 2>/dev/null || \
         grep -q '^#fi$' "$SCRIPT" 2>/dev/null; then
        ORIG="$SCRIPT.orig"
        if [ -f "$ORIG" ]; then
          ${pkgs.coreutils}/bin/cp "$ORIG" "$SCRIPT"
        else
          DEPLOY_DIR="$(${pkgs.coreutils}/bin/dirname "$(${pkgs.coreutils}/bin/dirname "$(${pkgs.coreutils}/bin/dirname "$SCRIPT")")")"
          if [ -f "$DEPLOY_DIR/files/extra/opt/wemeet/wemeetapp.sh" ]; then
            ${pkgs.coreutils}/bin/cp "$DEPLOY_DIR/files/extra/opt/wemeet/wemeetapp.sh" "$SCRIPT"
          fi
        fi
      fi

      if grep -q '^[[:space:]]*export WEMEET_XWAYLAND=1' "$SCRIPT" 2>/dev/null; then
        [ -f "$SCRIPT.orig" ] || ${pkgs.coreutils}/bin/cp "$SCRIPT" "$SCRIPT.orig"
        ${pkgs.gnused}/bin/sed -i \
          '/^if \[ "\$XDG_SESSION_TYPE" = "wayland" \];then/,/^fi$/{
             /^if /b
             /^fi$/b
             s/^/#/
           }' "$SCRIPT"
      fi
    fi

    # 清除全局 QT_QPA_PLATFORM=xcb
    if $FLATPAK override --show com.tencent.wemeet 2>/dev/null | grep -q 'QT_QPA_PLATFORM=xcb'; then
      $FLATPAK override --system --unset-env=QT_QPA_PLATFORM com.tencent.wemeet || true
    fi

    # 复制 X11 线程修复 .so 到 home（flatpak sandbox 内 /nix/store 不可访问）
    ${pkgs.coreutils}/bin/cp "${wemeetX11ThreadFix}/lib/libx11threadfix.so" "$FIX_DST"
    $FLATPAK override --system --env="LD_PRELOAD=$FIX_DST" com.tencent.wemeet || true
  '';
in
{
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
    wireplumber.enable = true;
  };

  hardware.bluetooth = {
    enable = true;
    powerOnBoot = true;
  };
  services.blueman.enable = true;

  services.upower.enable = true;
  services.power-profiles-daemon.enable = true;

  programs.appimage = {
    enable = true;
    binfmt = true;
  };

  services.flatpak.enable = flatpakCfg.enable;

  systemd.services.flatpak-setup = lib.mkIf flatpakCfg.enable {
    description = "Flatpak baseline setup (Flathub + default apps)";
    unitConfig.ConditionPathExists = "!${flatpakSetupStamp}";
    after = [
      "network-online.target"
      "dbus.service"
    ];
    wants = [ "network-online.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = pkgs.writeShellScript "flatpak-setup" ''
        set -euo pipefail
        stamp=${lib.escapeShellArg flatpakSetupStamp}
        ${pkgs.coreutils}/bin/mkdir -p "$(${pkgs.coreutils}/bin/dirname "$stamp")"
        ${lib.optionalString flatpakCfg.enableFlathub ''
          ${pkgs.flatpak}/bin/flatpak remote-add --system --if-not-exists \
            flathub https://flathub.org/repo/flathub.flatpakrepo
        ''}
        ${flatpakInstallScript}
        ${flatpakOverrideScript}
        ${pkgs.coreutils}/bin/touch "$stamp"
      '';
    };
  };

  systemd.services.flatpak-update = lib.mkIf (flatpakCfg.enable && flatpakCfg.autoUpdate.enable) {
    description = "Flatpak system update";
    after = [ "network-online.target" ];
    wants = [ "network-online.target" ];
    serviceConfig = lib.mkMerge [
      {
        Type = "oneshot";
        ExecStart = "${pkgs.flatpak}/bin/flatpak update --system -y --noninteractive";
      }
      (lib.mkIf (builtins.elem "com.tencent.wemeet" flatpakCfg.apps) {
        ExecStartPost = "${wemeetFixScript}";
      })
    ];
  };

  systemd.timers.flatpak-update = lib.mkIf (flatpakCfg.enable && flatpakCfg.autoUpdate.enable) {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = flatpakCfg.autoUpdate.onCalendar;
      Persistent = true;
      RandomizedDelaySec = flatpakCfg.autoUpdate.randomizedDelaySec;
    };
  };

  systemd.services.flatpak-fix-wemeet =
    lib.mkIf (flatpakCfg.enable && builtins.elem "com.tencent.wemeet" flatpakCfg.apps)
      {
        description = "Fix Tencent Wemeet Flatpak";
        after = [
          "flatpak-setup.service"
          "flatpak-update.service"
          "local-fs.target"
        ];
        wantedBy = [ "multi-user.target" ];
        serviceConfig = {
          Type = "oneshot";
          ExecStart = wemeetFixScript;
        };
      };

  boot.kernelModules = lib.mkAfter [ "v4l2loopback" ];
  boot.extraModulePackages = lib.mkAfter [ config.boot.kernelPackages.v4l2loopback ];
  boot.extraModprobeConfig = lib.mkAfter ''
    options v4l2loopback devices=1 video_nr=10 card_label="OBS Virtual Camera" exclusive_caps=1
  '';
}
