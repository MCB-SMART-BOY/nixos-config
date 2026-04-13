# 细节与联动

这份文档解释当前主线的关键联动点：TUI 怎么落盘、部署怎么执行、检查怎么兜底。

## 1. TUI 写回链

当前写回路径：

- `Packages` -> `home/users/<user>/managed/packages/*.nix`
- `Home` -> `home/users/<user>/managed/settings/desktop.nix`
- `Users` -> `hosts/<host>/managed/users.nix`
- `Hosts` -> `hosts/<host>/managed/network.nix` / `gpu.nix` / `virtualization.nix`

保存前规则：

- `Users` 和 `Hosts` 页都先做整机级校验，不再只校验当前页字段
- `Packages` 页删除陈旧组文件前，先确认它们属于受管文件
- `Home` / `Hosts` / `Packages` 写回都走统一的受管文件保护

## 2. 受管文件保护

`mcbctl` 现在不会再无条件覆盖 `managed/*.nix`。

当前规则：

1. 新写入文件带 `mcbctl-managed` 标记和校验摘要
2. 旧占位文件或旧受管格式允许迁移
3. 已带标记但内容被手改破坏的文件会被拒绝覆盖
4. `managed/packages/` 里混入非受管文件时，TUI 不会偷偷删掉它们

这意味着：

- 受管分片是 Rust 独占写回区域
- 手写逻辑应搬到 `default.nix`、`packages.nix` 或 `local.nix`

## 3. Host 配置校验

`Hosts` 页当前已经覆盖这几类校验：

- 缓存策略：`cacheProfile`、`customSubstituters`、`customTrustedPublicKeys`
- 代理模式：`proxyMode`、`proxyUrl`
- TUN / DNS：`tunInterface`、`tunInterfaces`、`enableProxyDns`、`proxyDns*`
- per-user TUN：接口映射、DNS 端口映射、基值、全局 DNS 冲突
- GPU：`mode`、`igpuVendor`、`prime.mode`、specialisation 模式合法性
- hybrid / GPU specialisation 的 PRIME Bus ID 前置条件
- 虚拟化：当前是结构化开关写回，复杂能力仍由 Nix 模块声明

这些校验尽量对齐：

- `modules/nix.nix`
- `modules/networking.nix`
- `modules/hardware/gpu.nix`

## 4. 部署执行链

部署主入口是 `mcb-deploy`，`mcbctl deploy` 只是转发。

Rust 侧负责：

1. 环境检查
2. 仓库完整性检查
3. 来源选择
4. 本地源准备或远端镜像重试
5. host / user / admin 选择
6. 结构化写回
7. `/etc/nixos` 备份与同步
8. `nixos-rebuild` 计划生成与执行
9. release 说明与发布

`Deploy` 页只在简单组合下直接执行；复杂来源或高级路径退回完整向导。

## 5. 桌面命令与 Noctalia

桌面命令现在都来自 Rust 二进制：

- `lock-screen`
- `niri-run`
- `noctalia-*`
- `electron-auto-gpu`
- `zed-auto-gpu`
- `flatpak-setup`
- `mcbctl terminal-action ...`
- `mcbctl screenshot-edit ...`

Noctalia / Niri 配置文件现在只保留静态命令字符串，不承载业务逻辑。

## 6. 仓库完整性

仓库检查统一走 Rust：

- `mcbctl repo-integrity`
- `mcbctl lint-repo`
- `mcbctl doctor`

主要检查项：

- 禁止 `.sh` / `.bash` / `.py`
- 禁止旧脚本目录和旧桥接层
- 禁止 `writeShell*`
- 禁止显式 `sh -c` / `bash -c` / `python -c` / `fish -c`
- 检查主线目录是否仍然完整

`flake check` 只负责调用这些 Rust 检查器和 Rust 构建，不再在 Nix builder 里承载项目脚本逻辑。
