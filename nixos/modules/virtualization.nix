{ ... }:

{
  virtualisation = {
    docker = {
      enable = true;
      storageDriver = "overlay2";
      autoPrune.enable = true;
    };
    libvirtd.enable = true;
  };

  programs.virt-manager.enable = true;
}
