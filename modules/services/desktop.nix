# 桌面服务：音频、图形驱动、AppImage、节能等。
# 主要影响桌面环境的"基础能力"。

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
in
{
  services.pipewire = {
    # 现代音频栈（替代 pulseaudio）
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
    wireplumber.enable = true;
  };

  # bluetooth
  hardware.bluetooth = {
    enable = true;
    powerOnBoot = true;
  };
  services.blueman.enable = true;

  # 电池状态（Noctalia 依赖 UPower 提供电池信息）
  services.upower.enable = true;

  # 电源管理（Noctalia 依赖 power-profiles-daemon）
  services.power-profiles-daemon.enable = true;

  programs.appimage = {
    # 允许直接运行 AppImage
    enable = true;
    binfmt = true;
  };

  # Flatpak 服务（桌面应用分发）
  services.flatpak.enable = flatpakCfg.enable;

  # Flatpak：默认配置 Flathub 远程仓库，并安装基础应用
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

  # Flatpak 自动更新（系统级）
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
        # 腾讯会议更新后重新 patch（如果更新覆盖了脚本）
        ExecStartPost = pkgs.writeShellScript "flatpak-update-wemeet-fix" ''
          set -euo pipefail
          wemeet_script="/var/lib/flatpak/app/com.tencent.wemeet/current/active/files/extra/opt/wemeet/wemeetapp.sh"

          if [ ! -f "$wemeet_script" ]; then
            exit 0
          fi

          # 1) unset WAYLAND_DISPLAY → Qt SIGABRT
          if grep -q '^[[:space:]]*unset WAYLAND_DISPLAY' "$wemeet_script" 2>/dev/null; then
            ${pkgs.gnused}/bin/sed -i \
              's/^[[:space:]]*unset WAYLAND_DISPLAY.*$/    # unset WAYLAND_DISPLAY  # patched by nixos-config/' \
              "$wemeet_script"
          fi

          # 2) export QT_QPA_PLATFORM=xcb → X11 多线程不安全
          if grep -q '^[[:space:]]*export QT_QPA_PLATFORM=xcb' "$wemeet_script" 2>/dev/null; then
            ${pkgs.gnused}/bin/sed -i \
              's/^[[:space:]]*export QT_QPA_PLATFORM=xcb.*$/    # export QT_QPA_PLATFORM=xcb  # patched by nixos-config/' \
              "$wemeet_script"
          fi

          # 3) export XDG_SESSION_TYPE=x11
          if grep -q '^[[:space:]]*export XDG_SESSION_TYPE=x11' "$wemeet_script" 2>/dev/null; then
            ${pkgs.gnused}/bin/sed -i \
              's/^[[:space:]]*export XDG_SESSION_TYPE=x11.*$/    # export XDG_SESSION_TYPE=x11  # patched by nixos-config/' \
              "$wemeet_script"
          fi

          # 4) export WEMEET_XWAYLAND=1
          if grep -q '^[[:space:]]*export WEMEET_XWAYLAND=1' "$wemeet_script" 2>/dev/null; then
            ${pkgs.gnused}/bin/sed -i \
              's/^[[:space:]]*export WEMEET_XWAYLAND=1.*$/    # export WEMEET_XWAYLAND=1  # patched by nixos-config/' \
              "$wemeet_script"
          fi

          # 5) 清除全局 flatpak override 强制设置的 QT_QPA_PLATFORM=xcb
          if ${pkgs.flatpak}/bin/flatpak override --show com.tencent.wemeet 2>/dev/null | grep -q 'QT_QPA_PLATFORM=xcb'; then
            ${pkgs.flatpak}/bin/flatpak override --system --unset-env=QT_QPA_PLATFORM com.tencent.wemeet || true
          fi
        '';
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

  # 腾讯会议 Flatpak 修复：
  # 1. wemeetapp.sh 在 Wayland 下 unset WAYLAND_DISPLAY → Qt SIGABRT
  # 2. 强制 QT_QPA_PLATFORM=xcb + WEMEET_XWAYLAND=1 → X11 多线程不安全 → SIGSEGV
  #    (libImSDK 线程调用 XSetInputFocus 导致 Display hook 指针损坏)
  # 修复：注释掉 Wayland→X11 强制回退逻辑，让 wemeet 走 Wayland 原生路径
  # 参见：https://github.com/flathub/com.tencent.wemeet/issues (wayland PR已合并)
  # 注意：Flatpak 每次更新会覆盖脚本，因此不使用 stamp file，每次启动都检查
  systemd.services.flatpak-fix-wemeet =
    lib.mkIf (flatpakCfg.enable && builtins.elem "com.tencent.wemeet" flatpakCfg.apps)
      {
        description = "Fix Tencent Wemeet Flatpak (Wayland→X11 fallback causes crash)";
        after = [
          "flatpak-setup.service"
          "flatpak-update.service"
          "local-fs.target"
        ];
        wantedBy = [ "multi-user.target" ];
        serviceConfig = {
          Type = "oneshot";
          ExecStart = pkgs.writeShellScript "flatpak-fix-wemeet" ''
            set -euo pipefail
            wemeet_script="/var/lib/flatpak/app/com.tencent.wemeet/current/active/files/extra/opt/wemeet/wemeetapp.sh"

            if [ ! -f "$wemeet_script" ]; then
              exit 0
            fi

            changed=false

            # 1) 注释掉 unset WAYLAND_DISPLAY（导致 Qt SIGABRT）
            if grep -q '^[[:space:]]*unset WAYLAND_DISPLAY' "$wemeet_script" 2>/dev/null; then
              ${pkgs.gnused}/bin/sed -i \
                's/^[[:space:]]*unset WAYLAND_DISPLAY.*$/    # unset WAYLAND_DISPLAY  # disabled by nixos-config/' \
                "$wemeet_script"
              changed=true
            fi

            # 2) 注释掉 export QT_QPA_PLATFORM=xcb（X11 多线程不安全，改用 Wayland 原生）
            if grep -q '^[[:space:]]*export QT_QPA_PLATFORM=xcb' "$wemeet_script" 2>/dev/null; then
              ${pkgs.gnused}/bin/sed -i \
                's/^[[:space:]]*export QT_QPA_PLATFORM=xcb.*$/    # export QT_QPA_PLATFORM=xcb  # disabled by nixos-config: use Wayland native/' \
                "$wemeet_script"
              changed=true
            fi

            # 3) 注释掉 export XDG_SESSION_TYPE=x11（保留原始 Wayland 会话类型）
            if grep -q '^[[:space:]]*export XDG_SESSION_TYPE=x11' "$wemeet_script" 2>/dev/null; then
              ${pkgs.gnused}/bin/sed -i \
                's/^[[:space:]]*export XDG_SESSION_TYPE=x11.*$/    # export XDG_SESSION_TYPE=x11  # disabled by nixos-config/' \
                "$wemeet_script"
              changed=true
            fi

            # 4) 注释掉 export WEMEET_XWAYLAND=1（不强制 XWayland）
            if grep -q '^[[:space:]]*export WEMEET_XWAYLAND=1' "$wemeet_script" 2>/dev/null; then
              ${pkgs.gnused}/bin/sed -i \
                's/^[[:space:]]*export WEMEET_XWAYLAND=1.*$/    # export WEMEET_XWAYLAND=1  # disabled by nixos-config/' \
                "$wemeet_script"
              changed=true
            fi

            # 5) 清除全局 flatpak override 强制设置的 QT_QPA_PLATFORM=xcb
            if ${pkgs.flatpak}/bin/flatpak override --show com.tencent.wemeet 2>/dev/null | grep -q 'QT_QPA_PLATFORM=xcb'; then
              ${pkgs.flatpak}/bin/flatpak override --system --unset-env=QT_QPA_PLATFORM com.tencent.wemeet || true
            fi
          '';
        };
      };

  # OBS 虚拟摄像头（v4l2loopback）
  boot.kernelModules = lib.mkAfter [ "v4l2loopback" ];
  boot.extraModulePackages = lib.mkAfter [ config.boot.kernelPackages.v4l2loopback ];
  boot.extraModprobeConfig = lib.mkAfter ''
    options v4l2loopback devices=1 video_nr=10 card_label="OBS Virtual Camera" exclusive_caps=1
  '';

  # GPU 相关配置已迁移到 modules/hardware/gpu.nix
}
