# 系统核心模块聚合入口：统一导入通用模块（不含桌面/虚拟化/游戏）。
# 桌面扩展请在 hosts/profiles/desktop.nix 里追加。

{ ... }:

{
  # 系统层功能模块统一在此聚合
  imports = [
    ./options.nix
    ./hardware/gpu.nix
    ./boot.nix
    ./networking.nix
    ./security.nix
    ./nix.nix
    ./packages.nix
    ./services/core.nix
  ];
}
