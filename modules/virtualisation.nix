{ config, pkgs, ... }:

{
  virtualisation = {
    incus.enable = true;
    podman = {
      enable = true;
      dockerCompat = true;
      dockerSocket.enable = true;
      defaultNetwork.settings.dns_enabled = true;
    };
    libvirtd = {
      enable = true;
      qemu = {
        package = pkgs.qemu_kvm;
        swtpm.enable = true;
        vhostUserPackages = with pkgs; [ virtiofsd ];
      };
      nss.enableGuest = true;
    };
    spiceUSBRedirection.enable = true;
    oci-containers.backend = "podman";
  };

  networking.nftables.enable = true;

  programs.virt-manager.enable = true;

  boot.kernelParams = [ "intel_iommu=on" "iommu=pt" ];
  #   [ "default_hugepagesz=1G" "hugepagesz=1G" ] ++
  #   [ "vfio-pci.ids=${lib.concatStringsSep} "," vfioIds}"];

  boot.extraModprobeConfig = ''options kvm_intel nested=1'';

  # booty.initrd.kernelModules = [ "vfio_pci" "vfio" "vfio_iommu_type1" ];
}
