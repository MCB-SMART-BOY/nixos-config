# 音乐播放工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableMusic")) (with pkgs; [
  ncspot # Spotify TUI 客户端
  mpd # 音乐守护进程
  ncmpcpp # MPD TUI 客户端
  playerctl # 媒体键控制
])
