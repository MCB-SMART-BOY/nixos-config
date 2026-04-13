# 用户模板

这里放的是新用户脚手架来源，不是已经启用的 Home Manager 用户目录。

可用模板：

- `home/templates/users/laptop/`
- `home/templates/users/server/`

`mcb-deploy` 会按主机类型优先从这里读取模板。

模板中最重要的部分：

- `packages.nix`
- `managed/`

其中：

- `packages.nix` 是手写长期软件结构的起点
- `managed/` 只是给 `mcbctl` 预留的受管落点

真正生效的用户入口仍然应该落在：

- `home/users/<user>/`
