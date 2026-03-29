# 系统监控与概览工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableShellTools")) (with pkgs; [
  btop # 现代资源监控 TUI
  bottom # 资源监控 TUI（另一个风格）
  fastfetch # 系统信息展示
  duf # 磁盘占用（df 替代）
  gdu # 磁盘空间分析
  dust # 目录体积分析
  procs # ps 增强版
])
