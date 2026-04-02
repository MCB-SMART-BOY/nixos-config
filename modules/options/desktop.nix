# 桌面兼容环境相关选项。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb.desktop.graphicsRuntime = {
    enable = mkOption {
      type = types.bool;
      default = true;
      description = "Export compatibility graphics runtime env for desktop sessions (LD_LIBRARY_PATH + Vulkan ICD path).";
    };

    libraryPath = mkOption {
      type = types.listOf types.str;
      default = [
        "/run/current-system/sw/lib"
        "/run/current-system/sw/share/nix-ld/lib"
        "/run/opengl-driver/lib"
        "/run/opengl-driver-32/lib"
      ];
      description = "Library search paths exported to LD_LIBRARY_PATH when graphics runtime compatibility env is enabled.";
    };

    vulkanIcdDir = mkOption {
      type = types.str;
      default = "/run/opengl-driver/share/vulkan/icd.d";
      description = "Default Vulkan ICD directory for VK_DRIVER_FILES and shell fallback expansion.";
    };
  };
}
