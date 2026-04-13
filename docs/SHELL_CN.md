# Shell 附录

这份文档不是主线说明，只是补充说明当前仓库里仍然保留的交互 shell 体验。

## 当前定位

- shell 只负责交互体验
- 系统级业务逻辑已经收口到 `mcbctl`
- Noctalia / Niri 只保留静态命令字符串

## 当前入口

`mcbnixos` 的交互配置仍在：

- `home/users/mcbnixos/config/fish/conf.d/`

`master` 的 zsh 参考副本仍在：

- `home/users/mcbnixos/config/zsh/REFERENCE.master.md`

## 和主线的关系

这些入口仍然存在，但它们不再定义仓库主线：

- `nrs` -> `mcbctl rebuild switch ...`
- `nrt` -> `mcbctl rebuild test ...`
- `nrb` -> `mcbctl rebuild boot ...`
- `nrc` -> `mcbctl build-host ...`

也就是说，fish 现在只是 Rust 主线的交互壳，不再承载部署、状态计算或仓库写回逻辑。

## 仍适合留在 shell 的内容

- `cd` 包装
- `mkcd`
- `fcd`
- `fe`
- 提示符、欢迎信息、交互别名

## 不应再从 shell 生长的内容

- 部署编排
- 仓库检查
- GPU 切换业务逻辑
- per-user TUN 路由逻辑
- managed 写回
