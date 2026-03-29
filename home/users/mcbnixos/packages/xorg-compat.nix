# Xwayland 兼容工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableXorgCompat")) (with pkgs; [
  xwayland # Xwayland 服务
  xwayland-satellite # Xwayland 集成辅助
  xorg.xhost # X11 访问控制工具
])
