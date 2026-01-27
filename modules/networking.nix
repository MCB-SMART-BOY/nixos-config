{ config, lib, options, pkgs, ... }:

let
  tunInterface = config.mcb.tunInterface;
  tunInterfaces = config.mcb.tunInterfaces;
  perUserTunEnabled = config.mcb.perUserTun.enable;
  perUserInterfaces = config.mcb.perUserTun.interfaces;
  perUserDnsRedirect = perUserTunEnabled && config.mcb.perUserTun.redirectDns;
  perUserDnsPorts = config.mcb.perUserTun.dnsPorts;
  userList =
    if config.mcb.users != [ ] then
      config.mcb.users
    else
      [ config.mcb.user ];
  tunInterfacesEffective = lib.unique (
    (lib.optionals (tunInterface != "") [ tunInterface ])
    ++ tunInterfaces
    ++ (lib.optionals perUserTunEnabled (lib.attrValues perUserInterfaces))
  );
  proxyUrl = config.mcb.proxyUrl;
  proxyMode = config.mcb.proxyMode;
  proxyServiceEnabled = proxyMode == "tun";
  proxyEnabled = proxyMode == "http" && proxyUrl != "";
  proxyDnsEnabled =
    proxyServiceEnabled
    && config.mcb.enableProxyDns;
  proxyDnsAddr = config.mcb.proxyDnsAddr;
  proxyDnsPort = config.mcb.proxyDnsPort;
  proxyDnsTarget =
    if proxyDnsPort == 53 then
      proxyDnsAddr
    else
      "${proxyDnsAddr}:${toString proxyDnsPort}";
  resolvedHasDns = lib.hasAttrByPath [ "services" "resolved" "dns" ] options;
  resolvedHasFallback = lib.hasAttrByPath [ "services" "resolved" "fallbackDns" ] options;
  resolvedExtraConfig = ''
    ${lib.optionalString (!resolvedHasDns && proxyDnsEnabled) "DNS=${proxyDnsTarget}"}
    ${lib.optionalString (!resolvedHasFallback && !proxyDnsEnabled) "FallbackDNS=223.5.5.5 1.1.1.1"}
  '';

  mkRouteService = idx: user:
    let
      iface = perUserInterfaces.${user};
      tableId = config.mcb.perUserTun.tableBase + idx;
      priority = config.mcb.perUserTun.priorityBase + idx;
      dnsPort = perUserDnsPorts.${user} or 0;
      dnsPortStr = toString dnsPort;
      dnsRedirectFlag = if perUserDnsRedirect then "1" else "0";
      ip = "${pkgs.iproute2}/bin/ip";
      iptables = "${pkgs.iptables}/bin/iptables";
      grep = "${pkgs.gnugrep}/bin/grep";
      sleep = "${pkgs.coreutils}/bin/sleep";
      seq = "${pkgs.coreutils}/bin/seq";
      id = "${pkgs.coreutils}/bin/id";
      routeScript = pkgs.writeShellScript "mcb-tun-route-${user}" ''
        set -euo pipefail

        uid="$(${id} -u ${user})"
        if [[ -z "$uid" ]]; then
          echo "User ${user} not found" >&2
          exit 1
        fi

        for _ in $(${seq} 1 50); do
          if ${ip} link show dev "${iface}" >/dev/null 2>&1; then
            break
          fi
          ${sleep} 0.2
        done

        if ! ${ip} link show dev "${iface}" >/dev/null 2>&1; then
          echo "Interface ${iface} not ready; skip route setup" >&2
          exit 0
        fi

        if ! ${ip} rule show | ${grep} -q "uidrange $uid-$uid.*lookup ${toString tableId}"; then
          ${ip} rule add priority ${toString priority} uidrange $uid-$uid lookup ${toString tableId}
        fi

        ${ip} route replace default dev "${iface}" table ${toString tableId}

        if [[ "${dnsRedirectFlag}" == "1" ]]; then
          if [[ "${dnsPortStr}" == "0" ]]; then
            echo "DNS redirect enabled but no port configured for ${user}" >&2
            exit 1
          fi
          if ! ${iptables} -t nat -C OUTPUT -p udp --dport 53 -m owner --uid-owner "$uid" -j REDIRECT --to-ports ${dnsPortStr} >/dev/null 2>&1; then
            ${iptables} -t nat -A OUTPUT -p udp --dport 53 -m owner --uid-owner "$uid" -j REDIRECT --to-ports ${dnsPortStr}
          fi
          if ! ${iptables} -t nat -C OUTPUT -p tcp --dport 53 -m owner --uid-owner "$uid" -j REDIRECT --to-ports ${dnsPortStr} >/dev/null 2>&1; then
            ${iptables} -t nat -A OUTPUT -p tcp --dport 53 -m owner --uid-owner "$uid" -j REDIRECT --to-ports ${dnsPortStr}
          fi
        fi
      '';
      stopScript = pkgs.writeShellScript "mcb-tun-route-${user}-stop" ''
        set -euo pipefail
        uid="$(${id} -u ${user} 2>/dev/null || true)"
        ${ip} route del default dev "${iface}" table ${toString tableId} >/dev/null 2>&1 || true
        if [[ -n "$uid" ]]; then
          ${ip} rule del uidrange $uid-$uid lookup ${toString tableId} >/dev/null 2>&1 || true
          if [[ "${dnsRedirectFlag}" == "1" ]]; then
            ${iptables} -t nat -D OUTPUT -p udp --dport 53 -m owner --uid-owner "$uid" -j REDIRECT --to-ports ${dnsPortStr} >/dev/null 2>&1 || true
            ${iptables} -t nat -D OUTPUT -p tcp --dport 53 -m owner --uid-owner "$uid" -j REDIRECT --to-ports ${dnsPortStr} >/dev/null 2>&1 || true
          fi
        fi
      '';
    in
    {
      description = "Per-user TUN routing (${user})";
      after = [
        "network-online.target"
        "clash-verge-service@${user}.service"
      ];
      partOf = [ "clash-verge-service@${user}.service" ];
      bindsTo = [ "clash-verge-service@${user}.service" ];
      wants = [
        "network-online.target"
        "clash-verge-service@${user}.service"
      ];
      unitConfig = {
        ConditionPathExists = "/sys/class/net/${iface}";
      };
      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
        ExecStart = routeScript;
        ExecStop = stopScript;
      };
    };
in
{
  assertions = lib.optionals perUserTunEnabled [
    {
      assertion = proxyMode == "tun";
      message = "mcb.perUserTun.enable requires mcb.proxyMode = \"tun\".";
    }
    {
      assertion = config.mcb.enableProxyDns == false;
      message = "per-user TUN does not support global proxy DNS. Set mcb.enableProxyDns = false.";
    }
    {
      assertion = lib.all (user: lib.hasAttr user perUserInterfaces) userList;
      message = "mcb.perUserTun.interfaces must define a TUN interface for each user in mcb.users.";
    }
    {
      assertion =
        lib.length (lib.unique (lib.attrValues perUserInterfaces))
        == lib.length (lib.attrValues perUserInterfaces);
      message = "mcb.perUserTun.interfaces must use unique interface names per user.";
    }
  ] ++ lib.optionals perUserDnsRedirect [
    {
      assertion = lib.all (user: lib.hasAttr user perUserDnsPorts) userList;
      message = "mcb.perUserTun.dnsPorts must define a port for each user when redirectDns is enabled.";
    }
    {
      assertion =
        lib.length (lib.unique (lib.attrValues perUserDnsPorts))
        == lib.length (lib.attrValues perUserDnsPorts);
      message = "mcb.perUserTun.dnsPorts must use unique ports per user.";
    }
  ];

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
      allowedTCPPorts =
        [
          22
        ];
      allowedUDPPorts = lib.optionals (proxyDnsEnabled && tunInterfacesEffective == [ ]) [ proxyDnsPort ];
      interfaces =
        (lib.optionalAttrs proxyDnsEnabled (lib.genAttrs tunInterfacesEffective (_: {
          allowedUDPPorts = [ proxyDnsPort ];
        })))
        // (lib.optionalAttrs proxyServiceEnabled {
          lo = {
            allowedTCPPorts = [
              7890
              9090
            ];
          };
        });
      trustedInterfaces =
        tunInterfacesEffective ++ [
          "tun+"
          "utun+"
          "docker0"
          "virbr0"
        ];
    };
  };

  systemd.services = lib.mkIf (proxyServiceEnabled && perUserTunEnabled) (
    lib.listToAttrs (lib.imap0 (idx: user: {
      name = "mcb-tun-route@${user}";
      value = mkRouteService idx user;
    }) userList)
  );

  systemd.paths = lib.mkIf (proxyServiceEnabled && perUserTunEnabled) (
    lib.listToAttrs (lib.imap0 (idx: user:
      let
        iface = perUserInterfaces.${user};
      in
      {
        name = "mcb-tun-route@${user}";
        value = {
          wantedBy = [ "multi-user.target" ];
          pathConfig = {
            PathExists = "/sys/class/net/${iface}";
            Unit = "mcb-tun-route@${user}.service";
          };
        };
      }) userList)
  );

  services.resolved =
    {
      enable = true;
    }
    // lib.optionalAttrs (resolvedHasDns && proxyDnsEnabled) {
      dns = [ proxyDnsTarget ];
    }
    // lib.optionalAttrs (resolvedHasFallback && !proxyDnsEnabled) {
      fallbackDns = [
        "223.5.5.5"
        "1.1.1.1"
      ];
    }
    // lib.optionalAttrs (resolvedExtraConfig != "") {
      extraConfig = resolvedExtraConfig;
    };
}
