# 用户脚本打包与 systemd 用户服务/定时器设置。
# 涉及 Waybar 模块与壁纸自动切换。
# 新手提示：scripts/ 下是原始脚本，这里负责“打包 + 安装 + 启动”。

{ pkgs, lib, ... }:

let
  # 将脚本包装为可执行程序（并注入依赖）
  mkScript =
    {
      name,
      runtimeInputs ? [ ],
    }:
    pkgs.writeShellApplication {
      inherit name runtimeInputs;
      text = builtins.readFile ./scripts/${name};
    };

  # 统一定义所有用户脚本（可在这里增删）
  scripts = {
    lock-screen = mkScript {
      name = "lock-screen";
      runtimeInputs = [ pkgs.swaylock ];
    };

    niri-run = mkScript {
      name = "niri-run";
    };

    wallpaper-random = mkScript {
      name = "wallpaper-random";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.findutils
        pkgs.procps
        pkgs.systemd
        pkgs.swaybg
      ];
    };

    waybar-flake-updates = mkScript {
      name = "waybar-flake-updates";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.gawk
        pkgs.git
        pkgs.jq
        pkgs.util-linux
      ];
    };

    waybar-gpu-mode = mkScript {
      name = "waybar-gpu-mode";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.findutils
      ];
    };

    waybar-net-speed = mkScript {
      name = "waybar-net-speed";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.gawk
        pkgs.gnugrep
        pkgs.iproute2
      ];
    };

    waybar-proxy-status = mkScript {
      name = "waybar-proxy-status";
      runtimeInputs = [ pkgs.systemd ];
    };
  };

  # 软链到 ~/.local/bin，方便手动执行
  mkBinLink = name: {
    source = "${scripts.${name}}/bin/${name}";
  };
in
{
  # 把脚本作为包安装到用户环境
  home.packages = lib.mkAfter (builtins.attrValues scripts);

  # 将脚本暴露为常用命令
  home.file.".local/bin/lock-screen" = mkBinLink "lock-screen";
  home.file.".local/bin/niri-run" = mkBinLink "niri-run";
  home.file.".local/bin/wallpaper-random" = mkBinLink "wallpaper-random";
  home.file.".local/bin/waybar-flake-updates" = mkBinLink "waybar-flake-updates";
  home.file.".local/bin/waybar-gpu-mode" = mkBinLink "waybar-gpu-mode";
  home.file.".local/bin/waybar-net-speed" = mkBinLink "waybar-net-speed";
  home.file.".local/bin/waybar-proxy-status" = mkBinLink "waybar-proxy-status";

  # 登录后随机设置壁纸
  systemd.user.services.wallpaper-random = {
    Unit = {
      Description = "Random wallpaper (swaybg)";
      After = [ "graphical-session.target" ];
      PartOf = [ "graphical-session.target" ];
      ConditionPathExistsGlob = "%t/wayland-*";
    };
    Service = {
      Type = "oneshot";
      ExecStart = "%h/.local/bin/wallpaper-random";
      Restart = "on-failure";
      RestartSec = 2;
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };

  # 定时切换壁纸（10 分钟一次）
  systemd.user.timers.wallpaper-random = {
    Unit = {
      Description = "Rotate wallpaper periodically";
      PartOf = [ "graphical-session.target" ];
    };
    Timer = {
      OnBootSec = "1m";
      OnUnitActiveSec = "10m";
      AccuracySec = "1m";
    };
    Install = {
      WantedBy = [ "graphical-session.target" ];
    };
  };
}
