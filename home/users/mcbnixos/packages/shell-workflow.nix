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
  oh-my-zsh # Zsh 插件框架
  zsh-autosuggestions # Zsh 自动建议
  zsh-syntax-highlighting # Zsh 语法高亮
  zsh-completions # 补全扩展
  fish # 另一套交互式 shell
])
