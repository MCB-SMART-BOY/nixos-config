# 主题与外观工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableTheming")) (with pkgs; [
  adwaita-icon-theme # GNOME 默认图标
  gnome-themes-extra # GNOME 主题补充
  papirus-icon-theme # Papirus 图标主题
  bibata-cursors # 鼠标光标主题
  catppuccin-gtk # GTK 主题
  nwg-look # GTK/图标主题切换 GUI
])
