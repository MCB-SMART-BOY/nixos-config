# mcbctl-managed: home-packages-aggregator
# mcbctl-checksum: f0f06469560c418171f5fd5d63c5499ff30d9a7447011c27e4a74516ef4db4d3
# 机器管理的用户软件入口（由 mcbctl 维护）。
# 说明：真正的软件组会按文件写入 ./packages/*.nix，这里只负责聚合导入。

{ lib, ... }:

let
  packageDir = ./packages;
  packageImports =
    if builtins.pathExists packageDir then
      builtins.map (name: packageDir + "/${name}") (
        lib.sort lib.lessThan (
          lib.filter (name: lib.hasSuffix ".nix" name) (builtins.attrNames (builtins.readDir packageDir))
        )
      )
    else
      [ ];
in
{
  imports = packageImports;
}
