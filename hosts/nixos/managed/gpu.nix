# mcbctl-managed: host-gpu
# mcbctl-checksum: a804b65a81b10c5e956e84059e1d801e860f0e1391a0fbfda467c00275b1a791
{ lib, ... }:

{
  mcb.hardware.gpu.mode = lib.mkForce "hybrid";
  mcb.hardware.gpu.igpuVendor = lib.mkForce "intel";
  mcb.hardware.gpu.prime = lib.mkForce {
    mode = "offload";
    intelBusId = "PCI:0:2:0";
    amdgpuBusId = null;
    nvidiaBusId = "PCI:1:0:0";
  };
  mcb.hardware.gpu.nvidia.open = lib.mkForce true;
  mcb.hardware.gpu.specialisations.enable = lib.mkForce true;
  mcb.hardware.gpu.specialisations.modes = lib.mkForce [ "igpu" "hybrid" "dgpu" ];
}
