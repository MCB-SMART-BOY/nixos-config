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
])
