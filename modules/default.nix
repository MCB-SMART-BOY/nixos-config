# 系统模块聚合入口：统一导入 modules/ 下的各功能模块。
# 大多数系统级功能都从这里被启用。

{ ... }:

{
  # 系统层功能模块统一在此聚合
  imports = [
    ./options.nix
    ./boot.nix
    ./networking.nix
    ./security.nix
    ./nix.nix
    ./packages.nix
    ./i18n.nix
    ./fonts.nix
    ./desktop.nix
    ./services.nix
    ./virtualization.nix
    ./gaming.nix
  ];
}
