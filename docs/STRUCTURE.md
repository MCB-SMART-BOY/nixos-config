# 项目结构

这份文档只描述当前分支的真实主线，不讨论已经删除的旧脚本路线。

最后更新：2026-05-03

## 顶层目录

```text
.
├── .github/workflows/
├── flake.nix
├── configuration.nix
├── mcbctl/
├── hosts/
├── modules/
├── home/
├── catalog/
├── pkgs/
└── docs/
```

边界固定如下：

- `.github/workflows/`：CI 质量门禁（fmt + clippy + test + flake check）和 release 资产构建
- `mcbctl/`：唯一业务逻辑实现层
- `hosts/`：真实主机、主机模板、主机 managed 分片
- `modules/`：NixOS 模块、`mcb.*` 选项和系统能力
- `home/`：真实用户、用户模板、静态程序配置、Home Manager 模块
- `catalog/`：TUI 元数据
- `pkgs/`：Rust 包和其他仓库内包的打包层

## 当前验证基线

```bash
cargo fmt --check --manifest-path mcbctl/Cargo.toml        ✅
cargo clippy --manifest-path mcbctl/Cargo.toml \
  --all-targets --all-features -- -D warnings              ✅
cargo test --manifest-path mcbctl/Cargo.toml               ✅ (456 tests)
nix flake check --option eval-cache false                  ✅
nix run .#mcbctl -- repo-integrity                         ✅
nix run .#mcbctl -- doctor                                 ✅
```

## `hosts/`

真实主机目录：

- `hosts/<host>/default.nix`
- `hosts/<host>/hardware-configuration.nix`
- `hosts/<host>/system.nix`
- `hosts/<host>/managed/`
- `hosts/<host>/local.auto.nix`
- `hosts/<host>/local.nix`
- `hosts/_support/hardware-configuration-eval.nix`

模板目录：

- `hosts/templates/laptop/`
- `hosts/templates/server/`

`hosts/<host>/managed/` 由 `mcbctl` 接管，当前固定分片为：

- `users.nix`
- `network.nix`
- `gpu.nix`
- `virtualization.nix`

人工长期逻辑不要写进这些分片；应写到 `default.nix` 或 `local.nix`。

`hosts/<host>/default.nix` 和模板入口当前会优先导入各自主机目录内的 `hardware-configuration.nix`；缺失时只在评估场景导入 `_support/hardware-configuration-eval.nix`。

## `modules/`

`modules/` 只负责声明能力，不负责项目流程编排。

主要区域：

- `modules/options/`：`mcb.*` 选项
- `modules/hardware/`：GPU 等硬件能力
- `modules/services/`：系统服务、代理服务、桌面辅助服务
- `modules/packages/`：系统共享包

这里允许 Nix 表达声明，不允许把项目逻辑藏进 shell 片段。

## `home/`

`home/` 负责用户会话结构：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/managed/`
- `home/users/<user>/local.auto.nix`
- `home/users/<user>/config/`
- `home/templates/users/`
- `home/modules/`

`home/users/<user>/managed/` 当前主要落点：

- `packages.nix`
- `packages/*.nix`
- `settings/default.nix`
- `settings/desktop.nix`

`config/` 里可以保留静态程序配置；项目业务逻辑不应继续从这里生长。

## `catalog/`

`catalog/` 只放 TUI 元数据：

- `catalog/packages/*.toml`
- `catalog/groups.toml`
- `catalog/home-options.toml`
- `catalog/workflows.toml`

它不承担写回逻辑、网络访问或状态计算。
它的作用不是重复 nixpkgs 官网搜索；当前主线里，`packages/*.toml` 更偏向项目自己的工作流、生命周期和 TUI 元数据层。

## `pkgs/`

`pkgs/` 只做打包和暴露：

- `pkgs/mcbctl/default.nix`
- 仓库自维护包如 `pkgs/zed/`、`pkgs/yesplaymusic/`、`pkgs/gridix/`

项目特有业务逻辑不应藏在 `pkgs/`。

## `mcbctl/`

`mcbctl/` 的层次固定为：

- `src/bin/`：命令入口
- `src/lib.rs`：共享底层工具和受管写入协议
- `src/domain/`：领域模型
- `src/store/`：I/O、渲染、持久化、环境探测
- `src/tui/`：状态和视图
- `src/repo.rs`：仓库完整性检查

按领域拆分的入口：

- `src/bin/control/`：`mcbctl`、`mcb-deploy`
- `src/bin/network/`：代理 / TUN 辅助命令
- `src/bin/desktop/`：桌面命令
- `src/bin/noctalia/`：Noctalia 状态与 GPU 模式命令
- `src/bin/update/`：上游 pin 检查和刷新

## 模板与 managed

两者用途不同：

- 模板：脚手架来源，只在创建新 host / 新 user 时使用
- managed：Rust/TUI 运行时写回落点

不要把模板当运行时状态，也不要把 `managed/` 当人工长期组织目录。

## 受管文件保护

本分支的 `managed/*.nix` 现在有统一约定：

- 新写入文件带 `mcbctl-managed` 标记和校验摘要
- `mcbctl migrate-managed` 负责显式升级可识别的旧受管文件
- `mcbctl extract-managed` 负责把残留手写模块移出 `managed/`
- `repo-integrity` / `lint-repo` 会检查受管文件的 marker、kind 和校验摘要
- 如果文件内容不再像受管文件，`mcbctl` 会拒绝覆盖

这条规则同样适用于 `managed/packages/*.nix` 的陈旧组文件删除。

## CI (新增于 2026-05-03)

- `.github/workflows/ci.yml`：PR 和 push 触发，跑 `cargo fmt --check`、`cargo clippy`、`cargo test`、`nix flake check`、`repo-integrity`、`doctor`
- `.github/workflows/release-mcbctl.yml`：手动或 tag 触发，构建并上传多平台预编译资产
