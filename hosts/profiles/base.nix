# 主机基础 profile：最小系统能力集合。

{ config, lib, ... }:

{
  # 基础 profile：系统核心功能，不含桌面相关模块
  imports = [
    ../../modules/default.nix
  ];

  # 仅混合显卡默认启用 GPU 特化，避免单显卡无意义切换
  mcb.hardware.gpu.specialisations = {
    enable = lib.mkDefault (config.mcb.hardware.gpu.mode == "hybrid");
    modes = lib.mkDefault [
      "igpu"
      "hybrid"
      "dgpu"
    ];
  };
}
