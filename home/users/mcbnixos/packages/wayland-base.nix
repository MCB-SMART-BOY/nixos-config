# Wayland 桌面基础工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableWaylandTools")) (with pkgs; [
  wl-clipboard # Wayland 剪贴板
  grim # Wayland 截图
  slurp # 区域选择（截图配套）
  swappy # 截图标注
  libnotify # 桌面通知接口
  fuzzel # Wayland launcher
  swayidle # 空闲管理
  niri # Wayland compositor（会话组件）
  pipewire # 多媒体管线
  brightnessctl # 背光控制
])
