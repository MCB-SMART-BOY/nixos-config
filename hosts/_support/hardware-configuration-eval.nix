# 仅用于仓库 / CI 评估的硬件回退模块。
# 真实部署应由仓库根目录或 /etc/nixos 提供真实 hardware-configuration.nix。

{ lib, ... }:

{
  fileSystems = lib.mkDefault {
    "/" = {
      device = "/dev/disk/by-label/__MCBCTL_EVAL_ONLY__";
      fsType = "ext4";
    };
  };
}
