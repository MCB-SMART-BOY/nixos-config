# 启动与内核相关设置：引导器、内核模块、sysctl 等。

{
  config,
  pkgs,
  lib,
  ...
}:

let
  # 固定到最新 longterm 稳定分支（当前为 6.18）；缺失时回退到发行版默认稳定内核。
  latestStableKernelPackages =
    if lib.hasAttrByPath [ "linuxKernel" "packages" "linux_6_18" ] pkgs then
      pkgs.linuxKernel.packages.linux_6_18
    else
      pkgs.linuxPackages;
in
{
  boot = {
    loader = {
      systemd-boot = {
        enable = true;
        editor = false;
        configurationLimit = 10;
      };
      efi.canTouchEfiVariables = true;
    };

    kernelPackages = lib.mkDefault latestStableKernelPackages;
    kernelModules = [
      "tun"
      "kvm-intel"
      "kvm-amd"
    ];

    kernel.sysctl = {
      "net.core.default_qdisc" = "fq";
      "net.ipv4.tcp_congestion_control" = "bbr";
      "net.ipv4.ip_forward" = lib.mkDefault 0;
      "net.ipv6.conf.all.forwarding" = lib.mkDefault 0;
      "net.netfilter.nf_conntrack_max" = 131072;
      "net.netfilter.nf_conntrack_tcp_timeout_established" = 1200;
      "net.core.rmem_max" = 16777216;
      "net.core.wmem_max" = 16777216;
      "net.ipv4.tcp_rmem" = "4096 87380 16777216";
      "net.ipv4.tcp_wmem" = "4096 65536 16777216";
    };
  };

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
