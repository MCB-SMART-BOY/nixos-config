{ ... }:

{
  networking = {
    hostName = "nixos";
    networkmanager = {
      enable = true;
    };
  };

  programs.clash-verge = {
    enable = true;
    serviceMode = true;
    tunMode = true;
  };
}
