# 主机基础 profile：最小系统能力集合。

{ ... }:

{
  # 基础 profile：系统核心功能，不含桌面相关模块
  imports = [
    ../../modules/options.nix
    ../../modules/boot.nix
    ../../modules/networking.nix
    ../../modules/security.nix
    ../../modules/nix.nix
    ../../modules/packages.nix
    ../../modules/services/core.nix
  ];
}
