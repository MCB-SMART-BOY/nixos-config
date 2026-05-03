# mcbctl-managed: host-network
# mcbctl-checksum: bc3ebb6aed27dc4d864dc8eeff3617115c7f2bd7171abc928737417a4a8bf1b7
{ lib, ... }:

{
  mcb.nix.cacheProfile = lib.mkForce "cn";
  mcb.nix.customSubstituters = lib.mkForce [ ];
  mcb.nix.customTrustedPublicKeys = lib.mkForce [ ];
  mcb.proxyMode = lib.mkForce "tun";
  mcb.proxyUrl = lib.mkForce "";
  mcb.tunInterface = lib.mkForce "Meta";
  mcb.tunInterfaces = lib.mkForce [ "Meta" "Mihomo" "clash0" ];
  mcb.enableProxyDns = lib.mkForce false;
  mcb.proxyDnsAddr = lib.mkForce "127.0.0.1";
  mcb.proxyDnsPort = lib.mkForce 53;
  mcb.perUserTun.enable = lib.mkForce true;
  mcb.perUserTun.compatGlobalServiceSocket = lib.mkForce true;
  mcb.perUserTun.redirectDns = lib.mkForce false;
  mcb.perUserTun.tableBase = lib.mkForce 1000;
  mcb.perUserTun.priorityBase = lib.mkForce 10000;
  mcb.perUserTun.interfaces = lib.mkForce {
    mcbnixos = "Meta";
  };
  mcb.perUserTun.dnsPorts = lib.mkForce {
    mcbnixos = 1053;
  };
}
