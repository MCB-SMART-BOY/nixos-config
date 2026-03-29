# 下载与桌面系统控制。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableSystemTools")) (with pkgs; [
  qbittorrent # BT 下载
  aria2 # 多协议下载
  yt-dlp # 视频下载
  gparted # 分区管理
  pavucontrol # 音频设备控制
])
