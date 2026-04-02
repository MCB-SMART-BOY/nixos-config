# Flatpak 集成相关选项。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb.flatpak = {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = "Enable Flatpak integration for this host.";
    };

    enableFlathub = mkOption {
      type = types.bool;
      default = true;
      description = "Add Flathub remote for system-wide Flatpak apps.";
    };

    apps = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Flatpak app IDs installed from Flathub (system-wide).";
    };

    overrides = {
      filesystem = mkOption {
        type = types.listOf types.str;
        default = [
          "xdg-desktop"
          "xdg-documents"
          "xdg-download"
          "xdg-music"
          "xdg-pictures"
          "xdg-public-share"
          "xdg-videos"
        ];
        description = "Default Flatpak filesystem overrides (system-wide).";
      };

      env = mkOption {
        type = types.attrsOf types.str;
        default = { };
        description = "Default Flatpak environment overrides (system-wide).";
      };

      extraArgs = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = "Extra flatpak override arguments applied system-wide.";
      };
    };

    autoUpdate = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Enable system Flatpak auto-updates.";
      };

      onCalendar = mkOption {
        type = types.str;
        default = "daily";
        description = "systemd OnCalendar value for Flatpak updates.";
      };

      randomizedDelaySec = mkOption {
        type = types.str;
        default = "1h";
        description = "Randomized delay for Flatpak updates.";
      };
    };
  };
}
