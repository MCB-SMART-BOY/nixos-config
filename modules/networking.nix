{ vars, lib, options, ... }:

let
  tunInterface = vars.tunInterface;
  proxyUrl = vars.proxyUrl;
  proxyEnabled = proxyUrl != "";
  resolvedHasDns = lib.hasAttrByPath [ "services" "resolved" "dns" ] options;
  resolvedHasFallback = lib.hasAttrByPath [ "services" "resolved" "fallbackDns" ] options;
  resolvedExtraConfig = ''
    ${lib.optionalString (!resolvedHasDns && proxyEnabled) "DNS=127.0.0.1"}
    ${lib.optionalString (!resolvedHasFallback) "FallbackDNS=223.5.5.5 1.1.1.1"}
  '';
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

  services.resolved =
    {
      enable = true;
    }
    // lib.optionalAttrs resolvedHasDns {
      dns = lib.optionals proxyEnabled [ "127.0.0.1" ];
    }
    // lib.optionalAttrs resolvedHasFallback {
      fallbackDns = [
        "223.5.5.5"
        "1.1.1.1"
      ];
    }
    // lib.optionalAttrs (resolvedExtraConfig != "") {
      extraConfig = resolvedExtraConfig;
    };
}
