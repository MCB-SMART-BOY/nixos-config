# 虚拟化与容器相关设置。

{ ... }:

{
  virtualisation = {
    docker = {
      # Docker 基础设置：overlay2 + 自动清理
      enable = true;
      storageDriver = "overlay2";
      autoPrune.enable = true;
    };
    # 启用 KVM/libvirt
    libvirtd.enable = true;
  };

  # 图形化管理 libvirt 虚拟机
  programs.virt-manager.enable = true;
}
