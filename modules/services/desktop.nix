# 桌面服务：音频、图形驱动、AppImage、节能等。
# 主要影响桌面环境的“基础能力”。

{ ... }:

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

  # GPU 相关配置已迁移到 modules/hardware/gpu.nix
}
