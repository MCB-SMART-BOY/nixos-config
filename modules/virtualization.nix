# 虚拟化与容器相关设置。

{ config, lib, ... }:

let
  cfg = config.mcb.virtualisation;
in
{
  options.mcb.virtualisation = {
    docker.enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable Docker daemon and related tooling.";
    };

    libvirtd.enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Enable libvirt/KVM stack.";
    };
  };

  config = {
    virtualisation = {
      docker = lib.mkIf cfg.docker.enable {
        # Docker 基础设置：overlay2 + 自动清理
        enable = true;
        storageDriver = "overlay2";
        autoPrune.enable = true;
      };
      libvirtd.enable = cfg.libvirtd.enable;
    };

    # 仅在 libvirt 启用时提供图形化管理器
    programs.virt-manager.enable = cfg.libvirtd.enable;
  };
}
