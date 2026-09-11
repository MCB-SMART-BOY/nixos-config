{ ... }:

{
  hardware.bluetooth = {
    enable = true;
    powerOnBoot = true;
  };

  services = {
    pipewire = {
      enable = true;
      pulse.enable = true;
      wireplumber.enable = true;
      alsa = {
        enable = true;
        support32Bit = true;
      };
    };

    blueman.enable = true;
    upower.enable = true;
    power-profiles-daemon.enable = true;
    printing.enable = true;

    flatpak.enable = true;
  };

  programs = {
    appimage = {
      enable = true;
      binfmt = true;
    };
    mtr.enable = true;
    gnupg.agent = {
      enable = true;
      enableSSHSupport = true;
    };
  };
}
