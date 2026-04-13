# Managed Packages

这个目录只给 `mcbctl` 的 `Packages` 页使用。

约定：

- 一个软件组对应一个 `.nix` 文件
- `managed/packages.nix` 只做聚合导入
- 这里的文件属于受管输出，不要放手写长期逻辑
- 新写回文件会带 `mcbctl-managed` 标记；如果内容不再像受管文件，TUI 会拒绝覆盖
