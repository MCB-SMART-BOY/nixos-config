# 用户脚本入口（Rust 版本）。
# 这里不再从 ./scripts 读取 Shell 脚本，而是直接安装 scripts-rs 包里的二进制。

{ pkgs, lib, ... }:

let
  scriptsRs = pkgs.callPackage ../../../pkgs/scripts-rs { };

  mkBinLink = name: {
    source = "${scriptsRs}/bin/${name}";
  };
in
{
  home.packages = lib.mkAfter [ scriptsRs ];

  home.file.".local/bin/lock-screen" = mkBinLink "lock-screen";
  home.file.".local/bin/niri-run" = mkBinLink "niri-run";
  home.file.".local/bin/noctalia-flake-updates" = mkBinLink "noctalia-flake-updates";
  home.file.".local/bin/noctalia-gpu-mode" = mkBinLink "noctalia-gpu-mode";
  home.file.".local/bin/noctalia-net-speed" = mkBinLink "noctalia-net-speed";
  home.file.".local/bin/noctalia-net-status" = mkBinLink "noctalia-net-status";
  home.file.".local/bin/noctalia-bluetooth" = mkBinLink "noctalia-bluetooth";
  home.file.".local/bin/noctalia-cpu" = mkBinLink "noctalia-cpu";
  home.file.".local/bin/noctalia-memory" = mkBinLink "noctalia-memory";
  home.file.".local/bin/noctalia-temperature" = mkBinLink "noctalia-temperature";
  home.file.".local/bin/noctalia-disk" = mkBinLink "noctalia-disk";
  home.file.".local/bin/noctalia-power" = mkBinLink "noctalia-power";
  home.file.".local/bin/noctalia-proxy-status" = mkBinLink "noctalia-proxy-status";
  home.file.".local/bin/steam-gamescope" = mkBinLink "steam-gamescope";
}
