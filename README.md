# NixOS 配置，`mcbctl` 主线

这个分支的主线已经固定：仓库的部署、发布、追新、桌面命令、TUI/CLI、managed 写回和仓库检查，只由 `mcbctl/` 里的 Rust 二进制加 Nix 声明完成。

仓库边界：

- `hosts/`：真实主机与主机模板
- `modules/`：系统层选项和能力声明
- `home/`：真实用户、用户模板、静态程序配置
- `catalog/`：Packages / Home 页元数据
- `pkgs/`：仓库自维护包和 Rust 打包
- `mcbctl/`：唯一业务逻辑实现层

不再存在 Shell / Python 业务主线，也不再保留旧脚本目录、旧入口脚本或 shell 包装流程。

## 主入口

```bash
nix run .#mcbctl
nix run .#mcbctl -- deploy
nix run .#mcb-deploy
```

常用检查：

```bash
nix run .#mcbctl -- repo-integrity
nix run .#mcbctl -- migrate-managed
nix run .#mcbctl -- extract-managed
nix run .#mcbctl -- migrate-hardware-config --host <host>
nix run .#mcbctl -- lint-repo
nix run .#mcbctl -- doctor
nix run .#update-upstream-apps -- --check
```

常用构建：

```bash
nix run .#mcbctl -- rebuild switch
nix run .#mcbctl -- rebuild test
nix run .#mcbctl -- rebuild boot
nix run .#mcbctl -- build-host --dry-run
```

## TUI 边界

- `Overview`：汇总当前 host、repo / doctor 健康、dirty 状态和推荐主动作
- `Apply`：承接当前 host 的默认应用路径，显示执行门槛、预览和高级 handoff
- `Inspect`：承接 `repo-integrity`、`doctor`、`flake check`、上游 pin 检查等检查动作
- `Packages`：写 `home/users/<user>/managed/packages/*.nix`
- `Home`：写 `home/users/<user>/managed/settings/desktop.nix`
- `Users`：写 `hosts/<host>/managed/users.nix`
- `Hosts`：写 `hosts/<host>/managed/network.nix`、`gpu.nix`、`virtualization.nix`
- `Actions`：过渡入口页；把历史杂糅动作按 `Inspect / Apply / Advanced` 归宿重新分发

这些页面保持当前职责边界，不通过 shell 函数补业务逻辑。`Actions` 当前仍保留，但定位已经变成迁移期入口，而不是长期主页面。

## Managed 写回规则

`mcbctl` 只写 `managed/` 分片，不直接改手写主体文件。

现在所有受管 `.nix` 文件都走统一的 Rust 写入协议：

- 新写入文件会带 `mcbctl-managed` 标记和校验摘要
- `mcbctl migrate-managed` 会把仓库里可识别的旧受管文件显式升级到新协议
- `mcbctl extract-managed` 会把残留在 `managed/` 里的手写模块抽到 `local.auto.nix` + `local-extracted/*.nix`
- `repo-integrity` / `lint-repo` 会把旧格式或错误 kind 的受管文件直接报出来
- TUI 只覆盖自己确认受管、且未被手改破坏的文件
- `Home` / `Users` / `Hosts` 保存时，还会检查同一 `managed` 子树里的兄弟分片
- 遇到陌生内容、损坏内容或 `managed/packages/` 中的非受管陈旧组文件，会直接拒绝覆盖或删除

运行时已经不再自动兼容旧 managed 格式；迁移只能通过显式命令完成。

手写长期逻辑应放在这些位置：

- `hosts/<host>/default.nix`
- `hosts/<host>/local.auto.nix` 仅用于 `extract-managed` 自动救援，不是长期手写入口
- `hosts/<host>/local.nix`
- `home/users/<user>/default.nix`
- `home/users/<user>/local.auto.nix` 仅用于 `extract-managed` 自动救援，不是长期手写入口
- `home/users/<user>/packages.nix`
- `home/users/<user>/local.nix`

不要把手写逻辑放进 `managed/`。

## 当前已保留能力

- `nix run .#mcbctl`
- `nix run .#mcbctl -- deploy`
- `nix run .#mcb-deploy`
- `Overview / Apply / Inspect / Packages / Home / Users / Hosts / Actions`
- `hosts/templates/` 与 `home/templates/users/`
- `mcb.hardware.gpu = igpu | hybrid | dgpu`
- `mcb.proxyMode = "tun" | "http" | "off"` 与 per-user TUN
- `lock-screen`
- `niri-run`
- 全部 `noctalia-*`
- `update-zed-source`
- `update-yesplaymusic-source`
- `update-upstream-apps`

## 验证

```bash
cargo fmt --check --manifest-path mcbctl/Cargo.toml
cargo clippy --manifest-path mcbctl/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path mcbctl/Cargo.toml
NIX_CONFIG='experimental-features = nix-command flakes' nix flake check --option eval-cache false
nix run .#mcbctl -- --help
nix run .#mcbctl -- migrate-managed --help
nix run .#mcbctl -- deploy --help
nix run .#mcb-deploy -- --help
nix run .#update-upstream-apps -- --check
```

`flake check` 现在会自动导入 [hardware-configuration-eval.nix](/home/mcbgaruda/projects/nixos-config/hosts/_support/hardware-configuration-eval.nix) 作为评估回退模块，所以仓库 / CI 没有真实 `hardware-configuration.nix` 也能安静评估；但 `switch` / `test` / `boot` 仍要求真实硬件配置文件存在。

真实硬件配置现在固定落在 `hosts/<host>/hardware-configuration.nix`；仓库根目录的 `hardware-configuration.nix` 已被视为旧路径，可用 `mcbctl migrate-hardware-config` 迁走。

发布主线现在默认使用当前 `mcbctl` 包版本作为 release tag，并会在创建 GitHub Release 后主动以该 tag 触发 [.github/workflows/release-mcbctl.yml](/home/mcbgaruda/projects/nixos-config/.github/workflows/release-mcbctl.yml)，由各平台 runner 构建并上传与该 release 对齐的 `mcbctl` 预编译资产。

## 文档

- [docs/USAGE.md](/home/mcbgaruda/projects/nixos-config/docs/USAGE.md)
- [docs/STRUCTURE.md](/home/mcbgaruda/projects/nixos-config/docs/STRUCTURE.md)
- [docs/DETAILS.md](/home/mcbgaruda/projects/nixos-config/docs/DETAILS.md)
- [docs/DEPLOY_AUDIT_CN.md](/home/mcbgaruda/projects/nixos-config/docs/DEPLOY_AUDIT_CN.md)
- [docs/UX_MAINLINE_CN.md](/home/mcbgaruda/projects/nixos-config/docs/UX_MAINLINE_CN.md)
- [docs/UX_SPEC_CN.md](/home/mcbgaruda/projects/nixos-config/docs/UX_SPEC_CN.md)
- [docs/NETWORK_CN.md](/home/mcbgaruda/projects/nixos-config/docs/NETWORK_CN.md)
- [docs/SHELL_CN.md](/home/mcbgaruda/projects/nixos-config/docs/SHELL_CN.md)
- [mcbctl/README.md](/home/mcbgaruda/projects/nixos-config/mcbctl/README.md)
