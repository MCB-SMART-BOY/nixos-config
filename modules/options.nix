# 自定义选项定义：集中声明 mcb.* 选项供其他模块读取。
# 例如代理、TUN、多用户等都由这里的 options 驱动。
# 新手提示：hosts/*/default.nix 中只需要改 mcb.*，其余模块会自动跟随。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb = {
    # 主用户（仅一个用户时用它；多用户时配合 users 列表）
    user = mkOption {
      type = types.str;
      default = "mcbnixos";
      description = "Primary system user.";
    };

    # 所有需要启用 Home Manager 的用户列表
    users = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "All system users managed by this host (Home Manager will be enabled for each).";
    };

    # CPU 厂商，用于选择正确的 KVM 模块（见 modules/boot.nix）
    cpuVendor = mkOption {
      type = types.enum [
        "intel"
        "amd"
      ];
      default = "intel";
      description = "CPU vendor for kernel module selection.";
    };

    # TUN 设备名（单一代理模式）
    tunInterface = mkOption {
      type = types.str;
      default = "";
      description = "Primary TUN interface name.";
    };

    # 额外允许的 TUN 设备名（白名单）
    tunInterfaces = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Additional TUN interface names to trust.";
    };

    # 代理模式：tun/http/off（影响 networking.nix 与 services/core.nix）
    proxyMode = mkOption {
      type = types.enum [
        "tun"
        "http"
        "off"
      ];
      default = "off";
      description = "Proxy mode: tun/http/off.";
    };

    # HTTP 代理地址（仅 proxyMode = http 时生效）
    proxyUrl = mkOption {
      type = types.str;
      default = "";
      description = "HTTP proxy URL (only used when proxyMode = \"http\").";
    };

    # 代理 DNS 设置（tun 模式下可强制本地 DNS）
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
      # 为每个用户独立走自己的 TUN 设备（高级用法）
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable per-user TUN routing with policy rules.";
      };

      # user -> TUN 接口名
      interfaces = mkOption {
        type = types.attrsOf types.str;
        default = { };
        description = "Per-user TUN interface mapping (user -> interface).";
      };

      # 是否将用户 DNS 53 重定向到本地端口
      redirectDns = mkOption {
        type = types.bool;
        default = false;
        description = "Redirect per-user DNS (uid-based) to local ports.";
      };

      # user -> DNS 监听端口
      dnsPorts = mkOption {
        type = types.attrsOf types.port;
        default = { };
        description = "Per-user DNS listen port mapping (user -> port).";
      };

      # 策略路由表 ID 起始值
      tableBase = mkOption {
        type = types.int;
        default = 1000;
        description = "Routing table base id for per-user rules.";
      };

      # ip rule 优先级起始值
      priorityBase = mkOption {
        type = types.int;
        default = 10000;
        description = "Priority base for per-user ip rules.";
      };
    };

    hardware = {
      nvidia = {
        enable = mkOption {
          type = types.bool;
          default = false;
          description = "Enable NVIDIA driver stack on this host (legacy switch; prefer mcb.hardware.gpu.mode).";
        };
      };
      gpu = {
        mode = mkOption {
          type = types.enum [
            "igpu"
            "hybrid"
            "dgpu"
          ];
          default = "igpu";
          description = "GPU topology: igpu (integrated only), hybrid (iGPU + NVIDIA dGPU), dgpu (NVIDIA only).";
        };

        igpuVendor = mkOption {
          type = types.enum [
            "intel"
            "amd"
          ];
          default = "intel";
          description = "Integrated GPU vendor for media acceleration packages and PRIME bus selection.";
        };

        prime = {
          mode = mkOption {
            type = types.enum [
              "offload"
              "sync"
              "reverseSync"
            ];
            default = "offload";
            description = "PRIME mode when using hybrid GPU (offload recommended for Wayland).";
          };

          intelBusId = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "Intel iGPU PCI bus id (e.g. PCI:0:2:0).";
          };

          amdgpuBusId = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "AMD iGPU PCI bus id (e.g. PCI:4:0:0).";
          };

          nvidiaBusId = mkOption {
            type = types.nullOr types.str;
            default = null;
            description = "NVIDIA dGPU PCI bus id (e.g. PCI:1:0:0).";
          };
        };

        nvidia = {
          open = mkOption {
            type = types.bool;
            default = false;
            description = "Use the NVIDIA open kernel module when supported.";
          };
        };

        specialisations = {
          enable = mkOption {
            type = types.bool;
            default = false;
            description = "Generate GPU specialisations (e.g. gpu-igpu/gpu-hybrid/gpu-dgpu) for easy switching.";
          };

          modes = mkOption {
            type = types.listOf (
              types.enum [
                "igpu"
                "hybrid"
                "dgpu"
              ]
            );
            default = [
              "igpu"
              "hybrid"
              "dgpu"
            ];
            description = "GPU modes to expose as specialisations.";
          };
        };
      };
    };
  };
}
