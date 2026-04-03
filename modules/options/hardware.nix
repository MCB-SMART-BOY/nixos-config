# GPU/NVIDIA 相关硬件选项。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb.hardware = {
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

      nvidia.open = mkOption {
        type = types.bool;
        default = false;
        description = "Use the NVIDIA open kernel module when supported.";
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
}
