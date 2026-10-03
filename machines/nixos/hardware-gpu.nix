{ config, ... }:

{
  services.xserver.videoDrivers = [ "nvidia" ];

  hardware = {
    graphics = {
      enable = true;
      enable32Bit = true;
    };
    nvidia = {
      package = config.boot.kernelPackages.nvidiaPackages.latest;
      modesetting.enable = true;
      open = true;
      nvidiaSettings = true;
      dynamicBoost.enable = true;
      powerManagement = {
        enable = true;
        finegrained = false;
      };
    };
    nvidia-container-toolkit.enable = true;
  };
}
