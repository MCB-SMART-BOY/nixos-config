# LSP 与格式化工具。

{ pkgs, ... }:

with pkgs; [
  nodePackages.typescript-language-server # TS/JS LSP
  nodePackages.prettier # 通用格式化器
  bash-language-server # Bash LSP
  pyright # Python 类型检查/LSP
  vscode-langservers-extracted # HTML/CSS/JSON/ESLint 等 LSP
  nixd # Nix LSP
  marksman # Markdown LSP
  taplo # TOML 工具/LSP
  yaml-language-server # YAML LSP
  lua-language-server # Lua LSP
  gopls # Go LSP
  nixfmt # Nix 格式化
  black # Python 格式化
  stylua # Lua 格式化
  shfmt # Shell 格式化
]
