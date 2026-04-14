# `mcbctl`

`mcbctl/` 是这个仓库唯一的业务逻辑实现层。

它负责的不是“Rust 版脚本集合”，而是当前真正的主线入口：

- `mcbctl`：TUI / CLI 总入口
- `mcb-deploy`：部署、发布、来源准备、同步与执行
- `noctalia-*`：桌面状态和 GPU 模式命令
- `lock-screen`、`niri-run`、`flatpak-setup`：桌面命令
- `update-*`：上游 pin 检查和刷新
- `repo-integrity` / `migrate-managed` / `extract-managed` / `migrate-hardware-config` / `lint-repo` / `doctor`：仓库检查与迁移
- `release-bundle`：生成 release 预编译资产

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

- `Overview`
- `Apply`
- `Inspect`
- `Packages`
- `Home`
- `Users`
- `Hosts`
- `Actions`

其中：

- `Overview` 负责健康总览、dirty 状态和推荐主动作
- `Apply` 负责当前 host 的默认应用路径和执行预览
- `Inspect` 负责健康详情和检查命令
- `Actions` 当前是过渡入口页，不是长期职责终点

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
- `extract-managed` 负责把残留在 `managed/` 里的手写内容抽到 `local.auto.nix` + `local-extracted/*.nix`
- `repo-integrity` / `lint-repo` 会检查 kind、marker 和校验摘要
- 被手改破坏的受管文件会被拒绝覆盖
- `Home` / `Users` / `Hosts` 保存时会连同同一 `managed` 子树的兄弟分片一起检查
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
nix run .#mcbctl -- extract-managed
nix run .#mcbctl -- migrate-hardware-config --host <host>
```

发布资产：

```bash
cargo run --manifest-path mcbctl/Cargo.toml --release --bin mcbctl -- release-bundle --target x86_64-unknown-linux-gnu --bin-dir mcbctl/target/release --out-dir dist --version vX.Y.Z
```

## 修改建议

- 改命令行为：先看 `src/bin/`
- 改 TUI 数据流：看 `src/domain/` + `src/store/` + `src/tui/state/`
- 改写回格式：看 `src/store/*` 和 `src/lib.rs`
- 改仓库规则：看 `src/repo.rs`
