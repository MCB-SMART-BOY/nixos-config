{ vars, lib, ... }:

let
  tunInterface = vars.tunInterface;
  proxyUrl = vars.proxyUrl;
  proxyEnabled = proxyUrl != "";
in
{
  networking = {
    networkmanager = {
      enable = true;
      dns = "systemd-resolved";
    };

    proxy = lib.mkIf proxyEnabled {
      default = proxyUrl;
      noProxy = "127.0.0.1,localhost,internal.domain";
    };

    firewall = {
      enable = true;
      checkReversePath = "loose";
      allowedTCPPorts = [
        22
        7890
        9090
      ];
      allowedUDPPorts = [ 53 ];
      trustedInterfaces =
        (lib.optionals (tunInterface != "") [ tunInterface ]) ++ [
          "tun+"
          "utun+"
          "docker0"
          "virbr0"
        ];
    };
  };

  services.resolved = {
    enable = true;
    dns = lib.optionals proxyEnabled [ "127.0.0.1" ];
    fallbackDns = [
      "223.5.5.5"
      "1.1.1.1"
    ];
  };
}
