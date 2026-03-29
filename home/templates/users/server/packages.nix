# 用户软件模板（server）：精简分组，适合作为新服务器用户起点。

{ lib, pkgs, ... }:

let
  # 按需开启附加组，避免默认模板过重。
  enableDevTools = false;
  enableDeepDiagnostics = false;

  # 服务器用户额外 CLI（系统层 shellTools 已提供 git/curl/wget/fd/rg/jq/yq 等）
  userCliExtras = with pkgs; [
    tmux
  ];

  # 运维额外工具（避免与系统层默认组重复）
  opsTools = with pkgs; [
    htop
    lsof
    rsync
    rclone
  ];

  # 可选：构建/开发
  devTools = with pkgs; [
    gnumake
    cmake
    gcc
    pkg-config
    neovim
    helix
  ];

  # 可选：深度网络诊断与抓包
  deepDiagnostics = with pkgs; [
    mtr
    nmap
    tcpdump
    traceroute
    iperf3
    ethtool
  ];
in
{
  home.packages = lib.concatLists [
    userCliExtras
    opsTools
    (lib.optionals enableDevTools devTools)
    (lib.optionals enableDeepDiagnostics deepDiagnostics)
  ];
}
