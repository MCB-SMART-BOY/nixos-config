# 自定义选项定义：集中声明 mcb.* 选项供其他模块读取。
# 所有 mcb.* 选项集中在此声明，模块文件只读取 config.mcb.*。

{ lib, config, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb = {
    # ── 用户与权限 ──────────────────────────────────────────────
    # 主用户（仅一个用户时用它；多用户时配合 users 列表）
    user = mkOption {
      type = types.str;
      default = "admin";
      description = "Primary system user.";
    };

    # 所有需要启用 Home Manager 的用户列表
    users = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "All system users managed by this host (Home Manager will be enabled for each).";
    };

    # 拥有管理员权限（wheel）的用户列表
    adminUsers = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Users granted admin privileges (wheel). Defaults to mcb.user when unset in host config.";
    };

    # 主机角色：影响默认用户组（桌面/服务器）
    hostRole = mkOption {
      type = types.enum [
        "desktop"
        "server"
      ];
      default = "desktop";
      description = "Host role used to derive default user group memberships.";
    };

    # 是否为托管用户默认开启 linger（允许用户服务在注销后继续运行）
    userLinger = mkOption {
      type = types.bool;
      default = false;
      description = "Enable user lingering for managed users.";
    };

    # ── 硬件 ────────────────────────────────────────────────────
    # CPU 厂商，用于选择正确的 KVM 模块（见 modules/boot.nix）
    cpuVendor = mkOption {
      type = types.enum [
        "intel"
        "amd"
      ];
      default = "intel";
      description = "CPU vendor for kernel module selection.";
    };

    # ── Nix 构建与缓存 ──────────────────────────────────────────
    nix = {
      cacheProfile = mkOption {
        type = types.enum [
          "cn"
          "global"
          "official-only"
          "custom"
        ];
        default = "cn";
        description = "Binary cache profile: cn/global/official-only/custom.";
      };

      customSubstituters = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = "Custom substituters used when mcb.nix.cacheProfile = \"custom\".";
      };

      customTrustedPublicKeys = mkOption {
        type = types.listOf types.str;
        default = [ ];
        description = "Custom trusted-public-keys used when mcb.nix.cacheProfile = \"custom\".";
      };
    };

    # ── 代理 ────────────────────────────────────────────────────
    # 代理模式：tun/http/off（影响 networking.nix）
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

    # ── Flatpak ─────────────────────────────────────────────────
    flatpak = {
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


    # 系统包组开关：控制 environment.systemPackages 中安装哪些包组
    packages = {
      enableNetwork = mkOption {
        type = types.bool;
        default = false;
        description = "Legacy switch: enable both network CLI and GUI packages.";
      };
      enableNetworkCli = mkOption {
        type = types.bool;
        default = false;
        description = "Install network/proxy CLI and service packages.";
      };
      enableNetworkGui = mkOption {
        type = types.bool;
        default = false;
        description = "Install network GUI tooling (applets, panels, bluetooth UI).";
      };
      enableShellTools = mkOption {
        type = types.bool;
        default = false;
        description = "Install CLI and shell utilities.";
      };
      enableWaylandTools = mkOption {
        type = types.bool;
        default = false;
        description = "Install Wayland-related tooling.";
      };
      enableSystemTools = mkOption {
        type = types.bool;
        default = false;
        description = "Install system utilities.";
      };
      enableInsecureTools = mkOption {
        type = types.bool;
        default = false;
        description = "Install insecure/legacy packages (disabled by default).";
      };
      enableTheming = mkOption {
        type = types.bool;
        default = false;
        description = "Install theming packages.";
      };
      enableXorgCompat = mkOption {
        type = types.bool;
        default = false;
        description = "Install Xorg compatibility tools.";
      };
      enableGeekTools = mkOption {
        type = types.bool;
        default = false;
        description = "Install common geek/debug/network tooling.";
      };
      enableMusic = mkOption {
        type = types.bool;
        default = false;
        description = "Install music players.";
      };
    };


    # ── 桌面与图形 ──────────────────────────────────────────────
    desktop = {
      graphicsRuntime = {
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
    };
  };
}
