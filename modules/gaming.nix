# 游戏相关支持与兼容设置。

{ config, lib, pkgs, ... }:

{
  # 可按需关闭游戏相关能力（比如服务器）
  options.mcb.system.enableGaming = lib.mkOption {
    type = lib.types.bool;
    default = true;
    description = "Enable system-level gaming features (Steam, gamemode).";
  };

  config = lib.mkIf config.mcb.system.enableGaming {
    # Steam + gamescope + 兼容层工具
    programs.steam = {
      enable = true;
      remotePlay.openFirewall = true;
      gamescopeSession.enable = true;
      extraCompatPackages = with pkgs; [
        mangohud
        gamemode
      ];
    };

    programs.gamemode.enable = true;
  };
}
