# 游戏相关工具。

{
  lib,
  pkgs,
  hostPkgEnabled,
  ...
}:

lib.optionals (!(hostPkgEnabled "enableGaming")) (with pkgs; [
  mangohud # 游戏性能叠层
  protonup-qt # Proton-GE 管理
  lutris # 游戏启动器/整合器
])
