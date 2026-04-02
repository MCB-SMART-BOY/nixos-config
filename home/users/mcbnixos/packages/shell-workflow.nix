# Shell 工作流工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableShellTools")) (with pkgs; [
  zoxide # 智能目录跳转
  starship # Shell Prompt
  direnv # 目录级环境变量管理
  fish # 默认交互式 shell
  tealdeer # tldr 客户端
  atuin # Shell 历史增强
  broot # 目录树/文件浏览器
  sd # sed 的现代替代
  zellij # 终端多路复用器
  ouch # 压缩/解压统一前端
])
