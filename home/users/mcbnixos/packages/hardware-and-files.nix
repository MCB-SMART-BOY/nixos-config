# 硬件信息与终端文件管理。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableShellTools")) (with pkgs; [
  lm_sensors # 传感器信息（温度/风扇）
  usbutils # USB 设备信息（lsusb）
  yazi # 终端文件管理器
])
