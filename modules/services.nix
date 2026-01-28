# 服务模块聚合入口：导入 core 与 desktop 服务。

{ ... }:

{
  # 服务分为 core / desktop 两部分，方便复用
  imports = [
    ./services/core.nix
    ./services/desktop.nix
  ];
}
