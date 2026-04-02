# TUI / 自动化工具专用主机入口。

{ lib, ... }:

{
  imports = [ ] ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
}
