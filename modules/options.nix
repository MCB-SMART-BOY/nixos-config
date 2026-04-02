# 自定义选项定义聚合入口：保持 `modules/default.nix -> ./options.nix` 不变，
# 但内部按用户、网络、桌面、硬件等领域拆到 `modules/options/`。

{ ... }:

{
  imports = [
    ./options/users.nix
    ./options/nix.nix
    ./options/networking.nix
    ./options/services.nix
    ./options/flatpak.nix
    ./options/desktop.nix
    ./options/hardware.nix
    ./options/assertions.nix
  ];
}
