# 桌面服务：音频、图形驱动、AppImage、节能等。
# 主要影响桌面环境的“基础能力”。

{ pkgs, lib, config, ... }:

{
  services.pipewire = {
    # 现代音频栈（替代 pulseaudio）
    enable = true;
    alsa.enable = true;
    pulse.enable = true;
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

  # F*ck you, Nv*dia(笑)

  hardware.graphics = {
    # 3D/视频硬件加速（Intel 默认）
    enable = true;
    enable32Bit = true;
    extraPackages = with pkgs; lib.optionals (config.mcb.cpuVendor == "intel") [
      intel-media-driver
      libvdpau-va-gl
    ];
  };
} // lib.optionalAttrs config.mcb.hardware.nvidia.enable {
  services.xserver.videoDrivers = [
    "modesetting"
    "nvidia"
  ];
  hardware.nvidia.open = true;
}
