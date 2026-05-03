# NixOS 配置，`mcbctl` 主线

[![ci](https://github.com/MCB-SMART-BOY/nixos-config/actions/workflows/ci.yml/badge.svg)](https://github.com/MCB-SMART-BOY/nixos-config/actions/workflows/ci.yml)

这个分支的主线已经固定：仓库的部署、发布、版本更新、桌面命令、TUI/CLI、managed 写回和仓库检查，统一由 `mcbctl/` 里的 Rust 二进制配合 Nix 声明完成。

仓库边界：

- `hosts/`：真实主机与主机模板
- `modules/`：系统层选项和能力声明
- `home/`：真实用户、用户模板、静态程序配置
- `catalog/`：Packages / Home / workflow 元数据
- `pkgs/`：仓库自维护包和 Rust 打包
- `mcbctl/`：唯一业务逻辑实现层

已移除所有 Shell / Python 业务代码、旧脚本目录及包装流程。

## 验证基线 (2026-05-03)

```bash
cargo fmt --check   ✅
cargo clippy        ✅ (0 warnings)
cargo test          ✅ (469 passed)
nix flake check     ✅ (真实 managed 数据 + 模板验证通过)
repo-integrity      ✅ (ok)
lint-repo           ✅ (ok)
doctor              ✅ (ok)
```

CI 自动验证：`.github/workflows/ci.yml`

## 主入口

```bash
nix run .#mcbctl                    # TUI
nix run .#mcbctl -- deploy          # 部署向导
nix run .#mcb-deploy                # 同上
nix run .#mcb-deploy -- release     # 创建 Release
```

## 常用命令

```bash
# 检查
nix run .#mcbctl -- repo-integrity
nix run .#mcbctl -- lint-repo
nix run .#mcbctl -- doctor

# 迁移
nix run .#mcbctl -- migrate-managed
nix run .#mcbctl -- extract-managed
nix run .#mcbctl -- migrate-hardware-config --host <host>

# 构建
nix run .#mcbctl -- rebuild switch
nix run .#mcbctl -- rebuild test
nix run .#mcbctl -- rebuild boot
nix run .#mcbctl -- build-host --dry-run

# 追新
nix run .#update-upstream-apps -- --check
nix run .#update-upstream-apps

# 发布
nix run .#mcbctl -- release-manifest
```

## TUI 区域

顶层固定为五个区域，`Tab` / `Shift-Tab` 切换。

### Overview

汇总当前 host、仓库健康、dirty 状态和推荐主动作。

- 左上 `Overview Summary` 按"当前判断 → 原因 → 最近结果 → 下一步 → 主动作"阅读
- `Enter` / `Space` / `a` / `p` 进入 `Apply` 预览
- `i` 进入 `Inspect`
- `r` / `d` / `R` 刷新健康项

### Edit

管理四个受管编辑页：`Packages`、`Home`、`Users`、`Hosts`。

- 顶栏用 `*` 标出有未保存修改的子页，`1`–`4` 切页
- 每个页面有自己的摘要、校验状态和保存反馈
- 窄屏时正文自动从双栏切换为上下堆叠
- 页脚显示当前页快捷键，`?` 打开完整帮助面板

写回落点：

| 页面 | 写入位置 |
|------|---------|
| `Packages` | `home/users/<user>/managed/packages/*.nix` |
| `Home` | `home/users/<user>/managed/settings/desktop.nix` |
| `Users` | `hosts/<host>/managed/users.nix` |
| `Hosts` | `hosts/<host>/managed/{network,gpu,virtualization}.nix` |

### Apply

当前 host 的默认部署路径。

- 左侧显示执行门槛（blocker / warning / handoff / info）和命令预览
- 右侧 `Current Selection` 显示建议、执行状态和 Advanced 交接提示
- `repo-integrity` / `doctor` 硬失败会直接阻塞执行
- `x` 执行当前默认路径，`Enter` 进入 `Advanced`

### Advanced

高级部署与维护的独立区域。

- 仓库维护路径：`Maintenance Summary + Maintenance Preview + Repository Context + Maintenance Detail`
- 完整向导路径：`Advanced Summary + Deploy Preview + Deploy Parameters + Deploy Wizard Detail`
- 参数独立于 `Apply`，`RemotePinned` 来源有独立的 `固定 ref` 输入
- `x` / `X` 执行当前高级动作，`b` 返回 `Apply`

### Inspect

健康详情和检查命令。

- 右侧 `Inspect Summary` 显示当前判断、最近结果、下一步、主动作
- `Health Details` 优先显示失败项和受管阻塞摘要
- `j` / `k` 选检查动作，`x` 执行，`r` / `d` / `R` 刷新

## Managed 写回规则

`mcbctl` 只写 `managed/` 分片，不直接改手写主体文件。

所有受管 `.nix` 文件走统一的写入协议：

- 新写入文件带 `mcbctl-managed` 标记和 SHA256 校验摘要
- `migrate-managed` 把可识别的旧受管文件升级到新协议
- `extract-managed` 把残留在 `managed/` 里的手写模块提取到 `local.auto.nix` + `local-extracted/*.nix`
- `repo-integrity` / `lint-repo` 检测旧格式、错误 kind 和损坏的受管文件
- TUI 只覆盖已确认受管且未被手改破坏的文件；保存时同时检查同目录下的兄弟分片
- 陌生内容、损坏内容或 `managed/packages/` 中的非受管陈旧文件会被拒绝覆盖或删除

运行时不再自动兼容旧 managed 格式；迁移必须通过显式命令完成。

### 手写逻辑放哪里

主机：
- `hosts/<host>/default.nix` — 主入口
- `hosts/<host>/local.nix` — 私有覆盖
- `hosts/<host>/local.auto.nix` — 仅用于 `extract-managed` 自动救援

用户：
- `home/users/<user>/default.nix` — 主入口
- `home/users/<user>/packages.nix` — 软件声明
- `home/users/<user>/local.nix` — 私有覆盖
- `home/users/<user>/local.auto.nix` — 仅用于 `extract-managed` 自动救援

不要把手写逻辑放进 `managed/`。

## 当前已保留能力

- `nix run .#mcbctl` (TUI 总入口)
- `nix run .#mcb-deploy` (部署向导)
- `Overview / Edit / Apply / Advanced / Inspect` (五区域 TUI)
- `hosts/templates/` 与 `home/templates/users/` (主机/用户模板)
- `mcb.hardware.gpu = igpu | hybrid | dgpu` (GPU 模式)
- `mcb.proxyMode = "tun" | "http" | "off"` 与 per-user TUN
- `lock-screen`、`niri-run` 等桌面命令
- 全部 `noctalia-*` (系统状态与 GPU 模式命令)
- `update-zed-source`、`update-yesplaymusic-source`、`update-upstream-apps`

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

`flake check` 会自动导入 `hosts/_support/hardware-configuration-eval.nix` 作为评估回退模块，所以仓库 / CI 没有真实 `hardware-configuration.nix` 也能安静评估；但 `switch` / `test` / `boot` 仍要求真实硬件配置文件存在。

真实硬件配置固定落在 `hosts/<host>/hardware-configuration.nix`；仓库根目录的 `hardware-configuration.nix` 是旧路径，用 `mcbctl migrate-hardware-config` 迁走。

发布主线默认使用当前 `mcbctl` 包版本作为 release tag，会在创建 GitHub Release 时附上一份 `release-manifest.json`，并以该 tag 触发 `.github/workflows/release-mcbctl.yml`，由各平台 runner 构建并上传预编译资产。

## 文档

- [docs/USAGE.md](docs/USAGE.md)
- [docs/STRUCTURE.md](docs/STRUCTURE.md)
- [docs/DETAILS.md](docs/DETAILS.md)
- [docs/ARCHITECTURE_BASELINE_CN.md](docs/ARCHITECTURE_BASELINE_CN.md)
- [docs/ROADMAP_MAINLINE_CN.md](docs/ROADMAP_MAINLINE_CN.md)
- [docs/AI_PROMPT_CHAIN_CN.md](docs/AI_PROMPT_CHAIN_CN.md)
- [docs/DEPLOY_AUDIT_CN.md](docs/DEPLOY_AUDIT_CN.md)
- [docs/UX_MAINLINE_CN.md](docs/UX_MAINLINE_CN.md)
- [docs/UX_SPEC_CN.md](docs/UX_SPEC_CN.md)
- [docs/NETWORK_CN.md](docs/NETWORK_CN.md)
- [docs/SHELL_CN.md](docs/SHELL_CN.md)
- [mcbctl/README.md](mcbctl/README.md)
