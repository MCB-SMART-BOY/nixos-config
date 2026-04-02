# TUI / 自动化工具专用入口。
# 约定：机器写入的用户级改动只落在 managed/，不要直接改手写 packages.nix / config/。

{ lib, ... }:

{
  imports = [
    ./packages.nix
  ]
  ++ lib.optional (builtins.pathExists ./settings/default.nix) ./settings/default.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
}
