# 主机基础 profile：最小系统能力集合。

{ lib, ... }:

{
  # 基础 profile：系统核心功能，不含桌面相关模块
  imports = [
    ../../modules/options.nix
    ../../modules/hardware/gpu.nix
    ../../modules/boot.nix
    ../../modules/networking.nix
    ../../modules/security.nix
    ../../modules/nix.nix
    ../../modules/packages.nix
    ../../modules/services/core.nix
  ];

  # 统一开启 GPU 特化（跨主机通用的安全默认）
  # 注意：不包含 hybrid，避免因缺少 busId 触发断言
  mcb.hardware.gpu.specialisations = {
    enable = lib.mkDefault true;
    modes = lib.mkDefault [
      "igpu"
      "dgpu"
    ];
  };
}
