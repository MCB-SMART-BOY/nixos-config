# 仅用于仓库 / CI 评估的硬件回退模块。
# 真实部署应由 hosts/<host>/hardware-configuration.nix 提供真实硬件配置。

{ lib, ... }:

{
  fileSystems = lib.mkDefault {
    "/" = {
      device = "/dev/disk/by-label/__MCBCTL_EVAL_ONLY__";
      fsType = "ext4";
    };
  };
}
