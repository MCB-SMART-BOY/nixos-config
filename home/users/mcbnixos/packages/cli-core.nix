# 日常 CLI 基础工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableShellTools")) (with pkgs; [
  git # 版本控制
  lazygit # Git TUI
  wget # 文件下载（非交互）
  curl # HTTP 调试/下载
  eza # ls 增强
  fd # find 替代，速度快
  fzf # 模糊搜索
  ripgrep # 全文搜索
  bat # cat 高亮版
  delta # git diff 美化
])
