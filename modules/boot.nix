{ pkgs, lib, vars, ... }:

let
  cpuVendor = if vars ? cpuVendor then vars.cpuVendor else "intel";
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
      systemd-boot = {
        enable = true;
        editor = false;
        configurationLimit = 10;
      };
      efi.canTouchEfiVariables = true;
    };

    kernelPackages = pkgs.linuxPackages_latest;
    kernelModules =
      [
        "tun"
      ]
      ++ lib.optional (kvmModule != null) kvmModule;

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
}
