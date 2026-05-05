# 桌面服务：音频、图形驱动、AppImage、节能等。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  flatpakCfg = config.mcb.flatpak;
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

  # wemeet 修复：wemeetapp.sh Wayland block + 全局 QT_QPA_PLATFORM override
  wemeetFixScript = pkgs.writeShellScript "wemeet-fix" ''
    set -euo pipefail
    SCRIPT="/var/lib/flatpak/app/com.tencent.wemeet/current/active/files/extra/opt/wemeet/wemeetapp.sh"
    FLATPAK="${pkgs.flatpak}/bin/flatpak"

    if [ -f "$SCRIPT" ]; then
      # 修复 shebang（之前的补丁可能误删了 #）
      ${pkgs.gnused}/bin/sed -i '1s/^!/#!/' "$SCRIPT"

      # 注释 Wayland→X11 回退 block 的 body 行，保留 if/fi 结构
      # 原始:
      #   if [ "$XDG_SESSION_TYPE" = "wayland" ];then
      #     if [ -f "/opt/x11-wayland/x11-ext.sh" ];then
      #       source /opt/x11-wayland/x11-ext.sh
      #     else
      #       export QT_QPA_PLATFORM=xcb
      #       export XDG_SESSION_TYPE=x11
      #       unset WAYLAND_DISPLAY
      #       export WEMEET_XWAYLAND=1
      #     fi
      #   fi
      # 目标: 保留 if/fi，body 全注释（if 体可以为空=合法）
      if grep -q 'export WEMEET_XWAYLAND=1' "$SCRIPT" 2>/dev/null || \
         grep -q '^#fi$' "$SCRIPT" 2>/dev/null; then
        # 从 .orig 恢复（如果存在），否则从 flatpak 仓库提取
        ORIG="$SCRIPT.orig"
        if [ -f "$ORIG" ]; then
          ${pkgs.coreutils}/bin/cp "$ORIG" "$SCRIPT"
        else
          # 从 flatpak ostree 签出原始文件
          DEPLOY_DIR="$(${pkgs.coreutils}/bin/dirname "$(${pkgs.coreutils}/bin/dirname "$(${pkgs.coreutils}/bin/dirname "$SCRIPT")")")"
          # 回退到 active 的原始状态
          if [ -f "$DEPLOY_DIR/files/extra/opt/wemeet/wemeetapp.sh" ]; then
            ${pkgs.coreutils}/bin/cp "$DEPLOY_DIR/files/extra/opt/wemeet/wemeetapp.sh" "$SCRIPT"
          fi
        fi
      fi

      # 备份并 patch（仅当仍有未注释的 WEMEET_XWAYLAND=1）
      if grep -q '^[[:space:]]*export WEMEET_XWAYLAND=1' "$SCRIPT" 2>/dev/null; then
        [ -f "$SCRIPT.orig" ] || ${pkgs.coreutils}/bin/cp "$SCRIPT" "$SCRIPT.orig"

        # 范围: outer if (^if ... wayland) → outer fi (^fi$)
        # 跳过 if 和 fi 行本身，其余全部注释
        ${pkgs.gnused}/bin/sed -i \
          '/^if \[ "\$XDG_SESSION_TYPE" = "wayland" \];then/,/^fi$/{
             /^if /b
             /^fi$/b
             s/^/#/
           }' "$SCRIPT"
      fi
    fi

    # 清除全局 QT_QPA_PLATFORM=xcb 对 wemeet 的影响
    if $FLATPAK override --show com.tencent.wemeet 2>/dev/null | grep -q 'QT_QPA_PLATFORM=xcb'; then
      $FLATPAK override --system --unset-env=QT_QPA_PLATFORM com.tencent.wemeet || true
    fi
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
        description = "Fix Tencent Wemeet Flatpak (Wayland native + clear X11 override)";
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
