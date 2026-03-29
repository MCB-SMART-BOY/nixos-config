# 媒体播放与阅读。

{ pkgs, ... }:

with pkgs; [
  nautilus # 文件管理器（GUI）
  mpv # 视频播放器
  vlc # 多媒体播放器
  imv # 图片查看器（Wayland 友好）
  zathura # PDF 阅读器（键盘友好）
]
