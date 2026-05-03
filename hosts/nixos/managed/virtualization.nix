# mcbctl-managed: host-virtualization
# mcbctl-checksum: 882d49d3e6601454a90c801076e37e404fca5bcc9ea99ba9d02b5e026d6f4808
{ lib, ... }:

{
  mcb.virtualisation.docker.enable = lib.mkForce false;
  mcb.virtualisation.libvirtd.enable = lib.mkForce false;
}
