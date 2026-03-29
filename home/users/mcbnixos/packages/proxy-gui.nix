# 网络代理 GUI。

{
  lib,
  pkgs,
  hostNetworkGuiEnabled,
  ...
}:

lib.optionals (!hostNetworkGuiEnabled) (with pkgs; [
  clash-nyanpasu # Clash GUI（MetaCubeX 生态）
  metacubexd # MetaCubeX 仪表盘/控制前端
])
