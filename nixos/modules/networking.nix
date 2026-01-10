{ ... }:

let
  vars = import ../../lib/vars.nix;
  tunInterface = vars.tunInterface;
in
{
  networking = {
    networkmanager = {
      enable = true;
      dns = "none";
    };

    nameservers = [
      "127.0.0.1"
      "223.5.5.5"
      "1.1.1.1"
    ];

    firewall = {
      enable = true;
      checkReversePath = "loose";
      allowedTCPPorts = [
        22
        7890
        9090
      ];
      allowedUDPPorts = [ 53 ];
      trustedInterfaces = [
        tunInterface
        "utun+"
        "docker0"
        "virbr0"
      ];
    };
  };

  services.resolved.enable = false;
}
