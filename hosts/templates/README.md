# 主机模板

这里放的是“复制源”，不是 flake 会直接部署的真实主机。

约定：

- `hosts/templates/laptop/`
  桌面 / 笔记本方向的完整主机样板
- `hosts/templates/server/`
  服务器方向的主机样板

使用方式：

1. 复制一个模板目录到 `hosts/<hostname>/`
2. 修改 `default.nix` 里的 `networking.hostName`、`mcb.user`、`mcb.users`
3. 生成并放入 `hosts/<hostname>/hardware-configuration.nix`
4. 再让 `flake.nix` 自动把它当成真实主机扫描

这些模板目录本身不会被 flake 扫描，也不应该直接作为部署目标。
