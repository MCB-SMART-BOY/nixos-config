# 代理、TUN 与 per-user TUN 相关选项。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb = {
    tunInterface = mkOption {
      type = types.str;
      default = "";
      description = "Primary TUN interface name.";
    };

    tunInterfaces = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Additional TUN interface names to trust.";
    };

    proxyMode = mkOption {
      type = types.enum [
        "tun"
        "http"
        "off"
      ];
      default = "off";
      description = "Proxy mode: tun/http/off.";
    };

    proxyUrl = mkOption {
      type = types.str;
      default = "";
      description = "HTTP proxy URL (only used when proxyMode = \"http\").";
    };

    enableProxyDns = mkOption {
      type = types.bool;
      default = true;
      description = "Force local DNS when proxyMode = \"tun\".";
    };

    proxyDnsAddr = mkOption {
      type = types.str;
      default = "127.0.0.1";
      description = "Local DNS address provided by the proxy.";
    };

    proxyDnsPort = mkOption {
      type = types.port;
      default = 53;
      description = "Local DNS port provided by the proxy.";
    };

    perUserTun = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable per-user TUN routing with policy rules.";
      };

      interfaces = mkOption {
        type = types.attrsOf types.str;
        default = { };
        description = "Per-user TUN interface mapping (user -> interface).";
      };

      redirectDns = mkOption {
        type = types.bool;
        default = false;
        description = "Redirect per-user DNS (uid-based) to local ports.";
      };

      dnsPorts = mkOption {
        type = types.attrsOf types.port;
        default = { };
        description = "Per-user DNS listen port mapping (user -> port).";
      };

      compatGlobalServiceSocket = mkOption {
        type = types.bool;
        default = true;
        description = "Keep /run/clash-verge-rev/service.sock as a compatibility symlink in per-user mode (points to mcb.user instance).";
      };

      tableBase = mkOption {
        type = types.int;
        default = 1000;
        description = "Routing table base id for per-user rules.";
      };

      priorityBase = mkOption {
        type = types.int;
        default = 10000;
        description = "Priority base for per-user ip rules.";
      };
    };
  };
}
