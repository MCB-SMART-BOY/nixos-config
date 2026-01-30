# 启动与内核相关设置：引导器、内核模块、sysctl 等。
# CPU 厂商相关的 KVM 模块由 mcb.cpuVendor 决定。

{
  config,
  pkgs,
  lib,
  ...
}:

let
  cpuVendor = config.mcb.cpuVendor;
  kvmModule =
    if cpuVendor == "amd" then
      "kvm-amd"
    else if cpuVendor == "intel" then
      "kvm-intel"
    else
      null;
in
{
  boot = {
    loader = {
      # systemd-boot 适合 UEFI 环境
      systemd-boot = {
        enable = true;
        editor = false;
        configurationLimit = 10;
      };
      efi.canTouchEfiVariables = true;
    };

    # 使用稳定内核（驱动兼容性更好）；需要最新内核时在 host 层覆盖
    kernelPackages = lib.mkDefault pkgs.linuxPackages;
    kernelModules = [
      "tun"
    ]
    ++ lib.optional (kvmModule != null) kvmModule;

    # 内核网络参数优化（BBR、队列、转发等）
    kernel.sysctl = {
      "net.core.default_qdisc" = "fq";
      "net.ipv4.tcp_congestion_control" = "bbr";
      "net.ipv4.ip_forward" = 1;
      "net.ipv6.conf.all.forwarding" = 1;
      "net.netfilter.nf_conntrack_max" = 131072;
      "net.netfilter.nf_conntrack_tcp_timeout_established" = 1200;
      "net.core.rmem_max" = 16777216;
      "net.core.wmem_max" = 16777216;
      "net.ipv4.tcp_rmem" = "4096 87380 16777216";
      "net.ipv4.tcp_wmem" = "4096 65536 16777216";
    };
  };

  # 有些环境不会自动创建 /dev/net/tun，确保 TUN 可用
  systemd.services.ensure-tun = {
    description = "Ensure TUN device node";
    wantedBy = [ "multi-user.target" ];
    serviceConfig.Type = "oneshot";
    script = ''
      ${pkgs.kmod}/bin/modprobe tun || true
      if [ ! -e /dev/net/tun ]; then
        ${pkgs.coreutils}/bin/mkdir -p /dev/net
        ${pkgs.coreutils}/bin/mknod /dev/net/tun c 10 200 || true
        ${pkgs.coreutils}/bin/chmod 0666 /dev/net/tun || true
      fi
    '';
  };
}
