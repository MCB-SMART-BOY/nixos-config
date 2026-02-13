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
  nvidiaEnabled = config.mcb.hardware.nvidia.enable || config.mcb.hardware.gpu.mode != "igpu";
  # Linux 新分支常先于闭源/混合 NVIDIA 模块适配；NVIDIA 主机默认回退到稳定内核。
  defaultKernelPackages = if nvidiaEnabled then pkgs.linuxPackages else pkgs.linuxPackages_latest;
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

    # 默认使用 latest；检测到 NVIDIA/hybrid/dgpu 时自动回退稳定分支以保证可构建。
    kernelPackages = lib.mkDefault defaultKernelPackages;
    kernelModules = [
      "tun"
    ]
    ++ lib.optional (kvmModule != null) kvmModule;

    # 内核网络参数优化（BBR、队列等）
    # 转发默认关闭：仅在确实充当网关时再在 host 层显式开启。
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
