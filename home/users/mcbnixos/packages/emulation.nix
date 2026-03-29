# 兼容层与 Windows 程序支持。

{ pkgs, ... }:

with pkgs; [
  wineWowPackages.stable # Wine（32/64 位兼容）
  winetricks # Wine 运行库安装脚本
]
