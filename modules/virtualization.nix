# 虚拟化与容器相关设置。
# 使用 NixOS 原生选项；在 local.nix 中以 mkForce 覆盖即可关闭。
#   virtualisation.docker.enable = lib.mkForce false;
#   virtualisation.libvirtd.enable = lib.mkForce false;

{ config, lib, ... }:

{
  config = {
    virtualisation = {
      docker = {
        enable = lib.mkDefault false;
        storageDriver = "overlay2";
        autoPrune.enable = true;
      };
      libvirtd.enable = lib.mkDefault false;
    };

    programs.virt-manager.enable = config.virtualisation.libvirtd.enable;
  };
}
