# 系统包模块聚合入口：选项、辅助套件与 systemPackages 逻辑拆分维护。

{ ... }:

{
  imports = [
    ./packages/options.nix
    ./packages/system.nix
  ];
}
