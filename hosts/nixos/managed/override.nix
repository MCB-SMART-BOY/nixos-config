# 兼容旧结构的主机覆盖入口。
# 现在主机级 managed 配置已经拆成 users.nix / network.nix / gpu.nix / virtualization.nix。
# 只有在这些分片都不存在时，managed/default.nix 才会回退导入本文件。

{ ... }:

{
}
