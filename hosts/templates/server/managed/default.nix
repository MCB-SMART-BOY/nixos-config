# mcbctl-managed: host-managed-default
# mcbctl-checksum: 4d18b54d362ce95946f4493dce8a687569d4f7ac2e176da951315b4de6155a0e
# TUI / 自动化工具专用主机入口。

{ lib, ... }:

{
  imports = [ ] ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;
}
