# 桌面服务：音频、图形驱动、AppImage、Flatpak 等。

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
  services.pipewire.enable = true;
  services.pipewire.alsa.enable = true;
  services.pipewire.pulse.enable = true;
  services.pipewire.wireplumber.enable = true;

  hardware.bluetooth.enable = true;
  hardware.bluetooth.powerOnBoot = true;
  services.blueman.enable = true;
  services.upower.enable = true;
  services.power-profiles-daemon.enable = true;

  programs.appimage.enable = true;
  programs.appimage.binfmt = true;

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
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${pkgs.flatpak}/bin/flatpak update --system -y --noninteractive";
    };
  };

  systemd.timers.flatpak-update = lib.mkIf (flatpakCfg.enable && flatpakCfg.autoUpdate.enable) {
    wantedBy = [ "timers.target" ];
    timerConfig = {
      OnCalendar = flatpakCfg.autoUpdate.onCalendar;
      Persistent = true;
      RandomizedDelaySec = flatpakCfg.autoUpdate.randomizedDelaySec;
    };
  };

  # 腾讯会议：nixpkgs 原生版本（非 flatpak），自带 Wayland 屏幕共享补丁和 X11 线程修复
  environment.systemPackages = [ pkgs.wemeet ];

  boot.kernelModules = lib.mkAfter [ "v4l2loopback" ];
  boot.extraModulePackages = lib.mkAfter [ config.boot.kernelPackages.v4l2loopback ];
  boot.extraModprobeConfig = lib.mkAfter ''
    options v4l2loopback devices=1 video_nr=10 card_label="OBS Virtual Camera" exclusive_caps=1
  '';
}
