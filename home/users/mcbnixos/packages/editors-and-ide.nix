# 编辑器与 IDE。

{ pkgs, ... }:

with pkgs; [
  neovim # 模态编辑器
  helix # 模态编辑器（现代配置）
  vscode-fhs # VS Code（FHS 兼容封装）
  isabelle # 定理证明环境
]
