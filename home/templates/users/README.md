# 用户模板

这里放的是新用户模板，不是实际启用的 Home Manager 用户目录。

约定：

- `home/templates/users/laptop/`
  桌面 / 笔记本用户的默认软件模板
- `home/templates/users/server/`
  服务器用户的默认软件模板

当前 `mcb-deploy` 会按主机类型优先从这里读取模板内容。

目前最重要的模板文件是：

- `packages.nix`

真正生效的用户入口仍然应该创建在：

- `home/users/<user>/`
