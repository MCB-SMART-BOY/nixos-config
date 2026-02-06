# 用户脚本打包与 systemd 用户服务设置。
# 主要用于 Noctalia 自定义按钮脚本。
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
    };

    wallpaper-random = mkScript {
      name = "wallpaper-random";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.findutils
      ];
    };

    niri-run = mkScript {
      name = "niri-run";
    };

    noctalia-flake-updates = mkScript {
      name = "noctalia-flake-updates";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.gawk
        pkgs.git
        pkgs.jq
        pkgs.util-linux
      ];
    };

    noctalia-gpu-mode = mkScript {
      name = "noctalia-gpu-mode";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.findutils
      ];
    };

    noctalia-net-speed = mkScript {
      name = "noctalia-net-speed";
      runtimeInputs = [
        pkgs.coreutils
        pkgs.gawk
        pkgs.gnugrep
        pkgs.iproute2
      ];
    };

    noctalia-proxy-status = mkScript {
      name = "noctalia-proxy-status";
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
  home.file.".local/bin/noctalia-flake-updates" = mkBinLink "noctalia-flake-updates";
  home.file.".local/bin/noctalia-gpu-mode" = mkBinLink "noctalia-gpu-mode";
  home.file.".local/bin/noctalia-net-speed" = mkBinLink "noctalia-net-speed";
  home.file.".local/bin/noctalia-proxy-status" = mkBinLink "noctalia-proxy-status";
}
