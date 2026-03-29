# 兼容旧结构的用户设置入口。
# 现在用户级 managed 设置已经拆成 settings/default.nix + desktop/session/mime 分片。
# 只有在 split 入口不存在时，managed/default.nix 才会回退导入本文件。

{ ... }:

{
}
