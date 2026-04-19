# NixOS 配置，`mcbctl` 主线

这个分支的主线已经固定：仓库的部署、发布、追新、桌面命令、TUI/CLI、managed 写回和仓库检查，只由 `mcbctl/` 里的 Rust 二进制加 Nix 声明完成。

仓库边界：

- `hosts/`：真实主机与主机模板
- `modules/`：系统层选项和能力声明
- `home/`：真实用户、用户模板、静态程序配置
- `catalog/`：Packages / Home / workflow 元数据
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
nix run .#mcbctl -- release-manifest
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

- `Overview`：汇总当前 host、repo / doctor 健康、dirty 状态和默认主路径入口；左上 `Overview Summary` 固定按“当前判断 -> 原因 -> 最近结果 -> 下一步 -> 主动作”阅读；`Enter / Space / a / p` 进入 `Apply` 预览，`i` 进入 `Inspect`
- `Edit`：承接 `Packages / Home / Users / Hosts` 四个受管编辑页；`Edit Pages` 顶栏会直接用 `*` 标出 dirty 子页，`Edit Workspace` 固定压成 `当前页/目标`、`Dirty`、`建议` 三行；`Home` / `Users` / `Hosts` 叶子标题统一收成目标导向短标题，例如 `Home (alice)`、`Users (demo)`、`Hosts (demo)`；窄宽下正文不再硬撑横向分栏，而是优先把主列表和摘要改成上下堆叠，`Packages` 会先把 `Packages Summary` 提到上方，再把列表与 `Selection` 下沉；`Home / Users / Hosts` 的摘要现在也会保留当前页 scoped 的 `最近结果 / 下一步`，让字段调整、保存成功和保存失败都留在页内可见；`Packages` 也不再把全局 `status` 直接塞回 `Selection`，而是改成稳定 `状态` 加 package-scoped 的 `最近结果 / 下一步`，把过滤、搜索、分组和保存反馈留在页内；进入窄宽后，`Home Summary` 会把长状态和写回说明压成 `用户 / 目标 / 聚焦 / 状态 / 写回` 这类短句，`Packages Summary` 会把源/用户、过滤、数量、当前流程、当前组收成稳定短行，`Packages` 列表标题会收成 `Packages`，列表项和 `Selection` 也会压成 `条目 / 类组 / 流程 / 已选 / 状态` 这类稳定摘要；默认 footer 现在统一先讲 `Edit/<Page>`、`1-4 子页`、`←/→ 目标`、`j/k 移动`，再补当前页主动作；`?` 帮助面板也先给共同骨架，再给当前页扩展键
- `Apply`：承接当前 host 的默认应用路径，显示执行门槛、预览和高级 handoff；左侧 `Apply Summary` 固定先看当前判断、最近结果、下一步、主动作，再看 `blocker / warning / handoff / info`，其中顶部四行和分类项都会优先压成短句；分类项显示“首个关键信号 + 额外项计数”，避免长状态文案堆回第一屏；`Apply Preview` 改成短标签 `目标 / 任务 / 来源 / 动作 / 升级 / 同步 / 执行`，避免长命令把默认路径第一屏挤爆；当宽度过窄时，`Apply` 会优先把 `Summary + Preview` 堆在上方，再把 `Current Selection + Apply Controls` 放到下半区；当高度继续变低时，也会进一步扩大左侧主路径区并压缩右侧次级区，避免低高度时先丢掉默认执行视野；右侧 `Current Selection` 只保留 `建议 / 执行状态 / 当前聚焦 / Advanced 交接提示`，并会在低高度下合并成 3 行或 2 行短表达；`Apply Controls` 也会同步压成短标签；`repo-integrity` / `doctor` 硬失败会直接进入 blocker，不再显示为“可直接执行”；右侧最后一行固定是 `区域切换 -> Enter 进入 Advanced`
- `Advanced`：承接高级部署与维护入口；现在已经是独立顶层区域，独占高级动作列表、仓库维护视图和完整 deploy wizard；进入后会显示推荐高级动作、进入原因和完成后的返回路径；左侧预览、中间上下文、右下角详情都会按当前高级动作自适应，仓库维护看 `Maintenance Summary + Maintenance Preview + Repository Context + Maintenance Detail`，完整向导看 `Advanced Summary + Deploy Preview + Deploy Parameters + Deploy Wizard Detail`；它自己的参数焦点和参数值都和 `Apply` 分离，`RemotePinned` 也有独立的 `固定 ref` 输入；进入 `mcb-deploy` 时会通过内部 handoff 参数显式带入 `host / mode / action / source / ref / upgrade`；动作列表会按推荐分组优先级自动排序；`x/X` 默认执行当前高级动作，`b` 返回 `Apply`
- `Inspect`：承接 `repo-integrity`、`doctor`、`flake check`、上游 pin 检查等检查动作；右侧 `Inspect Summary` 固定先显示当前判断、最近结果、下一步、主动作，并优先压成稳定短句，再看健康详情和命令细节；`Health Details` 会先压缩成优先失败项和首个受管阻塞摘要；当高度继续变低时，会优先保住 `Inspect Summary + Health Details`；`Command Detail` 只保留 `命令 / 状态 / 预览 / 分组 / 动作` 五行，命令预览也会去掉冗余前缀
- `Packages`：写 `home/users/<user>/managed/packages/*.nix`
- `Home`：写 `home/users/<user>/managed/settings/desktop.nix`
- `Users`：写 `hosts/<host>/managed/users.nix`
- `Hosts`：写 `hosts/<host>/managed/network.nix`、`gpu.nix`、`virtualization.nix`

当前顶层 shell 已经固定成 `Overview / Edit / Apply / Advanced / Inspect` 五个区域；历史 `Actions` 已经拆回各自归宿：`Inspect` / `Advanced` 直接持有页面动作，`Apply` 的 sync/rebuild 作为内部预览与执行链处理，不再保留独立 `Actions` 模块。
这些区域继续保持当前职责边界，不通过 shell 函数补业务逻辑。
默认页脚现在只保留当前页的短键位提示；详细热键统一收进 `?` 上下文帮助面板，`Esc` 只用于关闭帮助面板或取消当前输入模式。

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
- `Overview / Edit / Apply / Advanced / Inspect`
- `Edit` 内含 `Packages / Home / Users / Hosts`
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

发布主线现在默认使用当前 `mcbctl` 包版本作为 release tag，并会在创建 GitHub Release 时额外挂上一份机器可读的 `release-manifest.json`；随后主动以该 tag 触发 [.github/workflows/release-mcbctl.yml](/home/mcbgaruda/projects/nixos-config/.github/workflows/release-mcbctl.yml)，由各平台 runner 构建并上传与该 release 对齐的 `mcbctl` 预编译资产。

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
