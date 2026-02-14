# 桌面会话基础设置：niri、greetd、输入法环境变量、xdg-portal。
# 影响图形登录与 Wayland 应用的基础环境。
# 注意：输入法变量在 Home Manager 也会设置一份，保证 GUI 会话可见。

{
  config,
  pkgs,
  lib,
  ...
}:

{
  programs.niri.enable = true;
  programs.dconf.enable = true;
  programs.xwayland.enable = true;
  services.xserver.xkb.options = "ctrl:swapcaps";
  console.useXkbConfig = true;

  # 登录管理器：使用 greetd + tuigreet 启动 niri-session
  services.greetd = {
    enable = true;
    settings.default_session = {
      command = "${pkgs.tuigreet}/bin/tuigreet --time --greeting 'Welcome to NixOS' --asterisks --remember --remember-user-session --cmd niri-session";
      user = "greeter";
    };
  };

  systemd.services.greetd.serviceConfig = {
    Type = "idle";
    StandardInput = "tty";
    StandardOutput = "tty";
    StandardError = "journal";
    TTYReset = true;
    TTYVHangup = true;
    TTYVTDisallocate = true;
  };

  environment.sessionVariables = {
    # Wayland 优化与输入法环境变量
    # dGPU + NVIDIA + Wayland 路径下部分 Electron/Chromium 应用渲染不稳定；
    # 在 dGPU 模式回退到 X11（仅影响 Ozone 应用），igpu/hybrid 继续走 Wayland。
    NIXOS_OZONE_WL = if config.mcb.hardware.gpu.mode == "dgpu" then "0" else "1";
    # 同步收敛到稳定路径：Firefox/Electron 在 dGPU 模式禁用 Wayland 后端。
    MOZ_ENABLE_WAYLAND = if config.mcb.hardware.gpu.mode == "dgpu" then "0" else "1";
    ELECTRON_OZONE_PLATFORM_HINT = if config.mcb.hardware.gpu.mode == "dgpu" then "x11" else "auto";
    # 供用户脚本和 desktop wrapper 读取当前 GPU 拓扑（igpu/hybrid/dgpu）。
    MCB_GPU_MODE = config.mcb.hardware.gpu.mode;
    GTK_IM_MODULE = "fcitx";
    QT_IM_MODULE = "fcitx";
    SDL_IM_MODULE = "fcitx";
    GLFW_IM_MODULE = "fcitx";
    XMODIFIERS = "@im=fcitx";
    XIM_SERVERS = "fcitx";
    # 会话级 Vulkan ICD 发现路径，覆盖 GUI 应用与非交互启动场景
    VK_DRIVER_FILES = lib.mkDefault "/run/opengl-driver/share/vulkan/icd.d";
    # 不强制覆盖，避免吞掉其他模块追加的桌面数据目录
    XDG_DATA_DIRS = lib.mkDefault (
      lib.concatStringsSep ":" [
        "/run/opengl-driver/share"
        "/run/opengl-driver-32/share"
        "/run/current-system/sw/share"
        "/var/lib/flatpak/exports/share"
      ]
    );
  };

  # 会话变量是主策略；这里仅补充兼容：把 ICD 目录展开为 VK_ICD_FILENAMES 列表。
  environment.shellInit = ''
    if [ -z "''${VK_ICD_FILENAMES-}" ]; then
      icd_dir="''${VK_DRIVER_FILES:-/run/opengl-driver/share/vulkan/icd.d}"
      if [ -d "$icd_dir" ]; then
        vk_icd_files=""
        for file in "$icd_dir"/*.json; do
          [ -e "$file" ] || continue
          if [ -n "$vk_icd_files" ]; then
            vk_icd_files="$vk_icd_files:$file"
          else
            vk_icd_files="$file"
          fi
        done
        if [ -n "$vk_icd_files" ]; then
          export VK_ICD_FILENAMES="$vk_icd_files"
        fi
      fi
    fi
  '';

  xdg.portal = {
    # Portal 让截图/文件选择等桌面能力可用
    enable = true;
    # niri 的屏幕共享默认用 GNOME portal；Screencast 单独切到 wlr（可调格式兼容性）
    wlr.enable = true;
    wlr.settings = {
      screencast = {
        # 多 GPU / DMABUF 兼容性问题时强制线性 modifier
        force_mod_linear = true;
      };
    };
    extraPortals = [
      pkgs.xdg-desktop-portal-gnome
      pkgs.xdg-desktop-portal-gtk
    ];
    config = {
      common.default = [
        "gnome"
        "gtk"
      ];
      common."org.freedesktop.impl.portal.ScreenCast" = [ "wlr" ];
      niri.default = [
        "gnome"
        "gtk"
      ];
      niri."org.freedesktop.impl.portal.ScreenCast" = [ "wlr" ];
    };
  };
}
