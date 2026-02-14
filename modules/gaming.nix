# 游戏相关支持与兼容设置。
{
  config,
  lib,
  pkgs,
  ...
}:

let
  steamNativePackage = pkgs.steam.override {
    # niri/Xwayland black window workaround from niri wiki (native Steam path).
    extraArgs = "-system-composer";
    extraProfile = ''
      # Steam runtime can misread directory-style Vulkan vars from session env.
      unset VK_DRIVER_FILES
      unset VK_ICD_FILENAMES
    '';
  };
in
{
  # 可按需关闭游戏相关能力（比如服务器）
  options.mcb.system.enableGaming = lib.mkOption {
    type = lib.types.bool;
    default = true;
    description = "Enable system-level gaming features (Steam, gamemode).";
  };

  config = lib.mkIf config.mcb.system.enableGaming {
    # Steam（原生）+ 兼容层工具
    programs.steam = {
      enable = true;
      package = steamNativePackage;
      remotePlay.openFirewall = true;
      gamescopeSession.enable = false;
      extraCompatPackages = with pkgs; [
        mangohud
        gamemode
      ];
    };

    programs.gamemode.enable = true;
  };
}
