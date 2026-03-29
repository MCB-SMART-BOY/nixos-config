# Managed Packages

这个目录给 `mcbctl` 的 Packages 页面使用。

约定：

- 一个软件组对应一个 `.nix` 文件
- `managed/packages.nix` 只做聚合导入
- 这里的文件可以由 TUI 重写，不要放手写长期逻辑
