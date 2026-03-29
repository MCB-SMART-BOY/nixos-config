# 桌面服务：音频、图形驱动、AppImage、节能等。
# 主要影响桌面环境的“基础能力”。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  flatpakCfg = config.mcb.flatpak;
  scriptsRs = pkgs.callPackage ../../pkgs/scripts-rs { };
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
  flatpakSetupConfig = pkgs.writeText "flatpak-setup.json" (
    builtins.toJSON {
      stamp = flatpakSetupStamp;
      managed_apps_file = flatpakManagedApps;
      enable_flathub = flatpakCfg.enableFlathub;
      apps = flatpakCfg.apps;
      filesystem = flatpakCfg.overrides.filesystem;
      envs = map (key: "${key}=${flatpakCfg.overrides.env.${key}}") (
        builtins.attrNames flatpakCfg.overrides.env
      );
      extra_args = flatpakCfg.overrides.extraArgs;
    }
  );
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

  # 笔记本电源管理（桌面建议开启）
  services.tlp.enable = true;

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
    path = [
      pkgs.coreutils
      pkgs.flatpak
    ];
    after = [
      "network-online.target"
      "dbus.service"
    ];
    wants = [ "network-online.target" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${scriptsRs}/bin/flatpak-setup-rs --config ${flatpakSetupConfig}";
    };
  };

  # Flatpak 自动更新（系统级）
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

  # OBS 虚拟摄像头（v4l2loopback）
  boot.kernelModules = lib.mkAfter [ "v4l2loopback" ];
  boot.extraModulePackages = lib.mkAfter [ config.boot.kernelPackages.v4l2loopback ];
  boot.extraModprobeConfig = lib.mkAfter ''
    options v4l2loopback devices=1 video_nr=10 card_label="OBS Virtual Camera" exclusive_caps=1
  '';

  # GPU 相关配置已迁移到 modules/hardware/gpu.nix
}
