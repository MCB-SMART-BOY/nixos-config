# 蓝牙工具。

{
  lib,
  pkgs,
  hostNetworkGuiEnabled,
  ...
}:

lib.optionals (!hostNetworkGuiEnabled) (with pkgs; [
  bluez # 蓝牙协议栈核心工具集
  bluez-tools # 蓝牙命令行辅助工具
  blueman # 蓝牙图形管理器
])
