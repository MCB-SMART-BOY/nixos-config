# 动漫、漫画与相关内容工具。

{
  lib,
  pkgs,
  ...
}:

(with pkgs; [
  kazumi # 动漫聚合客户端
  mangayomi # 漫画/视频聚合
  bilibili # 哔哩哔哩客户端
  ani-cli # 终端动漫工具
  mangal # 漫画下载/阅读 CLI
  venera # 漫画下载/阅读 GUI
])
