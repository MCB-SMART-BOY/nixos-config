# 桌面服务：音频、图形驱动、AppImage、节能等。
# 主要影响桌面环境的“基础能力”。

{ config, lib, ... }:

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

  # 笔记本电源管理（桌面建议开启）
  services.tlp.enable = true;

  programs.appimage = {
    # 允许直接运行 AppImage
    enable = true;
    binfmt = true;
  };

  # OBS 虚拟摄像头（v4l2loopback）
  boot.kernelModules = lib.mkAfter [ "v4l2loopback" ];
  boot.extraModulePackages = lib.mkAfter [ config.boot.kernelPackages.v4l2loopback ];
  boot.extraModprobeConfig = lib.mkAfter ''
    options v4l2loopback devices=1 video_nr=10 card_label="OBS Virtual Camera" exclusive_caps=1
  '';

  # GPU 相关配置已迁移到 modules/hardware/gpu.nix
}
