{ pkgs, ... }:

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

  fileSystems = {
    "/var/lib/lxcfs" = {
      noCheck = true;
      options = [ "noauto" ];
    };
    "/var/lib/incus/devices".options = [ "noauto" ];
    "/var/lib/incus/guestapi".options = [ "noauto" ];
    "/var/lib/incus/shmounts".options = [ "noauto" ];
  };

  networking.nftables.enable = true;

  programs.virt-manager.enable = true;

  boot.kernelParams = [
    "intel_iommu=on"
    "iommu=pt"
  ];

  boot.extraModprobeConfig = "options kvm_intel nested=1";

}
