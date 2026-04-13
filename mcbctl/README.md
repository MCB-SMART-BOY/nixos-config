# `mcbctl`

`mcbctl/` 是这个仓库唯一的业务逻辑实现层。

它负责的不是“Rust 版脚本集合”，而是当前真正的主线入口：

- `mcbctl`：TUI / CLI 总入口
- `mcb-deploy`：部署、发布、来源准备、同步与执行
- `noctalia-*`：桌面状态和 GPU 模式命令
- `lock-screen`、`niri-run`、`flatpak-setup`：桌面命令
- `update-*`：上游 pin 检查和刷新
- `repo-integrity` / `migrate-managed` / `lint-repo` / `doctor`：仓库检查与受管协议维护

## 分层

- `src/bin/`：命令入口
- `src/lib.rs`：共享底层工具、受管写入协议
- `src/domain/`：领域模型
- `src/store/`：I/O、渲染、持久化、环境探测
- `src/tui/`：状态和视图
- `src/repo.rs`：仓库完整性规则

按领域拆开的入口：

- `src/bin/control/`
- `src/bin/network/`
- `src/bin/desktop/`
- `src/bin/noctalia/`
- `src/bin/update/`

## TUI 页面

`mcbctl` 当前负责这些页面：

- `Packages`
- `Home`
- `Users`
- `Hosts`
- `Deploy`
- `Actions`

写回位置：

- `Packages` -> `home/users/<user>/managed/packages/*.nix`
- `Home` -> `home/users/<user>/managed/settings/desktop.nix`
- `Users` -> `hosts/<host>/managed/users.nix`
- `Hosts` -> `hosts/<host>/managed/network.nix` / `gpu.nix` / `virtualization.nix`

## 保存与保护

`mcbctl` 现在不会再盲写 `managed/`。

当前写回协议：

- 新写入文件带 `mcbctl-managed` 标记和校验摘要
- `migrate-managed` 负责显式升级可识别的旧占位和旧受管格式
- `repo-integrity` / `lint-repo` 会检查 kind、marker 和校验摘要
- 被手改破坏的受管文件会被拒绝覆盖
- `managed/packages/` 中的非受管陈旧文件不会被自动删除

这条规则的目的不是“更严格”，而是避免 TUI 静默吃掉人工内容。

## 校验

TUI 和 CLI 当前会用到这些校验：

- 主机用户结构校验
- 网络 / TUN / per-user TUN 校验
- GPU 模式与 PRIME Bus ID 校验
- 仓库禁项与主线目录校验

尽量对齐：

- `modules/nix.nix`
- `modules/networking.nix`
- `modules/hardware/gpu.nix`

## 开发

```bash
cd mcbctl
cargo check
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

仓库根目录验证：

```bash
NIX_CONFIG='experimental-features = nix-command flakes' nix flake check --option eval-cache false
nix run .#mcbctl -- repo-integrity
nix run .#mcbctl -- migrate-managed
```

## 修改建议

- 改命令行为：先看 `src/bin/`
- 改 TUI 数据流：看 `src/domain/` + `src/store/` + `src/tui/state/`
- 改写回格式：看 `src/store/*` 和 `src/lib.rs`
- 改仓库规则：看 `src/repo.rs`
