# 细节与联动

这份文档解释当前主线的关键联动点：TUI 怎么落盘、部署怎么执行、检查怎么兜底。

如果要继续沿部署主线做审计和补测，下一阶段的交互矩阵与命令语义基线见 [DEPLOY_AUDIT_CN.md](./DEPLOY_AUDIT_CN.md)。

## 0. `Overview` 首页

当前顶层 shell 已经从旧的 8 页循环收成 5 个固定区域：

- `Overview`
- `Edit`
- `Apply`
- `Advanced`
- `Inspect`

其中：

- `Edit` 内部继续承接 `Packages / Home / Users / Hosts`
- `Edit` 外壳现在会显示可见的子导航和区域摘要，不再只靠 footer 记忆当前子页
- `Edit Pages` 顶栏现在会直接用 `*` 标出 dirty 子页，减少为了找未保存页来回切换
- `Edit` 的区域摘要现在固定压成三行：`当前页/目标`、`Dirty`、`建议`；推荐顺序仍然优先指出当前页 dirty，其次指出当前页受管保护，再回退到其他 dirty/guard 页
- `Home` / `Users` / `Hosts` 的叶子标题已经统一收成目标导向短标题，例如 `Home (alice)`、`Users (demo)`、`Hosts (demo)`，不再显示 `Home Settings`、`Users Model`、`Host Override` 这类历史过渡文案
- `Packages` 的左侧摘要标题也已收成 `Packages Summary`，不再继续保留 `Package Context` 这类过渡命名
- 当终端宽度变窄时，`Edit` 正文不再硬撑左右分栏：`Home / Users / Hosts` 会优先显示主列表，再把摘要下沉；`Packages` 会先把 `Packages Summary` 提到上方，再把列表与 `Selection` 放到下半区；更窄时三块继续顺序堆叠，优先保住标题和第一屏定位
- `Home Summary` 在窄宽或低高下不再原样灌入长状态和长说明；会优先压成 `用户 / 目标 / 聚焦 / 状态 / 受管保护 / 写回` 这类稳定短句，并把 `已接入` 列表压成短标签
- `Home / Users / Hosts` 的摘要现在会复用现有 `UiFeedbackScope::Home / Users / Hosts`，把当前页最近一次字段调整、保存成功、保存失败对应的 `最近结果 / 下一步` 保留在页内；这仍然只复用现有 feedback，不新增第二套 dirty / blocker / handoff 真值
- `Packages` 现在也会复用现有 `UiFeedbackScope::Packages`：过滤、搜索、分组、workflow 批量加入和保存结果都会进入页内 `最近结果 / 下一步`；右侧 `Selection.status` 同时改回稳定状态，不再把带 `下一步` 的全局 legacy status 直接塞进去
- `Packages Summary` 在窄宽或低高下不再继续逐条堆完整过滤和统计字段；会把 `源/用户`、`过滤`、`数量`、`当前流程`、`当前组` 压成稳定短行，并只在非 tight 模式保留目标目录和当前组落点
- `Packages` 列表在窄宽下会把标题收成 `Packages`，列表项从 `[x] Hello (cli, -> misc)` 这类长表达压成 `[x] Hello [cli/misc]` 或更短的名称行；右侧 `Selection` 也会收成 `条目 / 类组 / 组 / 流程 / 已选 / 状态` 这类一屏可扫的摘要，不再继续把整套工作流、分组和说明长段落原样堆进窄列
- `Edit` 的短 footer 现在不再按页各说各话，而是统一成“共同骨架 + 当前页主动作”：先显示 `Edit/<Page>`、`1-4 子页`、`←/→ 目标`、`j/k 移动`，再补 `Enter/Space 动作`、`h/l/Enter 调整` 或 `Enter 编辑` 这类当前页主动作
- `Edit` 的 `?` 帮助面板也改成统一阅读顺序：先看 `Edit Workspace` 和当前页主列表/摘要，再看共同骨架，最后看当前页主动作与扩展键；这样 `Packages / Home / Users / Hosts` 不再各自重复一套不同结构的说明页
- 默认页脚现在只保留当前页的短键位摘要；详细热键统一收进 `?` 帮助面板，避免继续把整套操作说明塞在长 footer 里
- `Advanced` 现在已经是独立顶层区域，也有独立的 `Page::Advanced` 叶子和按键分支；完整向导的参数焦点和参数值都已经从 `Apply` 分离出来，并会在进入 `mcb-deploy` 时通过内部 handoff 参数显式传递 `host / mode / action / source / ref / upgrade`；当来源是 `RemotePinned` 时，TUI 会额外显示并校验 `固定 ref` 输入，缺失时不会启动 deploy wizard；进入后会先显示按动作路径分流的区域摘要，仓库维护现在使用独立的 `Maintenance Summary`，不再复用 deploy 参数或 Apply 告警；左侧预览、中间上下文和右下角详情也都会按当前高级动作切换成“仓库维护”或“完整部署”两种视图，动作列表则按推荐分组自动重排
- 历史 `Actions` 已经降成迁移期内部模块，不再作为顶层区域显示

当前 `Dashboard` 已经演进成 `Overview` 过渡视图，不再只是静态路线图文本。

当前行为：

- 启动时先缓存一次 `repo-integrity` 结果
- `doctor` 初始保持未刷新，避免 TUI 启动时自动跑更多外部命令
- `Overview` 页内快捷键：
  - `r` 刷新 `repo-integrity`
  - `d` 刷新 `doctor`
  - `R` 同时刷新两者
- 刷新结果会进入 `Overview` 缓存，同时写一条状态提示到 TUI 状态栏文案

这意味着首页现在已经能回答：

- 当前 host 是否可用
- 当前组合能否直接 Apply
- 当前仓库结构是否健康
- 当前宿主环境是否具备部署能力
- 如果不能 Apply，阻塞点是在仓库、宿主环境还是当前配置

当前 `Overview` 主动作已经收成单一路径：

- `Enter / Space` 固定进入 `Apply` 预览
- `a / p` 也只作为进入 `Apply` 预览的快捷键，不再从 `Overview` 直接执行
- `i` 仍然显式打开 `Inspect`
- `?` 打开或关闭当前页帮助面板；帮助面板打开时会拦住页面动作，`Esc` 只负责关闭帮助面板
- dirty、健康失败、受管保护阻塞、handoff 不再抢走 `Overview` 主动作，而是继续显示在首页和 `Apply` 预览里
- 从 `Overview` 进入 `Apply` 时，会把右侧控件焦点重置回 `目标主机`，不再沿用历史 `区域切换` 焦点把用户直接带向 `Advanced`
- `Overview` 左上 `Overview Summary` 现在和 `Apply / Inspect` 统一成同一阅读顺序：先看 `当前判断`，再看补充 `原因`、`最近结果`、`下一步`，最后看 `主动作`
- `Overview` 右下 `Apply Snapshot` 现在只保留 `默认来源 / 默认动作 / 同步` 三项，不再重复 `Apply Summary` 已经承接的状态分类
- `Health` 首行现在固定用 `host-config: ...` 做目标主机配置摘要；`Dirty State` 固定压成 `状态 / 待保存 / 优先保存` 三行，不再继续堆解释句
- `Current Context` 现在优先压成成对字段：`目标/当前`、`用户/权限`，配合更紧的顶行高度，避免短终端里首页第一屏被上下文挤爆

也就是说，首页不再负责自动分流到别的页，而是先把用户送进默认主路径。

## 0.1 `Inspect` 页

`Inspect` 现在是检查动作的固定归宿页，不再把 repo 健康和检查命令混在 `Actions` 里。

当前行为：

- 展示 `repo-integrity` 和 `doctor` 的缓存结果
- 右侧固定先显示 `Inspect Summary`，统一给出当前判断、最近结果、下一步、主动作，再往下看 `Health Details` 和 `Command Detail`
- `Inspect Summary` 顶部四行现在也优先压成短句，像 `当前应复查 repo-integrity`、`flake check 已完成`、`先看 repo-integrity，需要时按 x`、`先看健康详情` 会优先出现，避免第一屏继续被检查说明长句塞满
- `Health Details` 现在和 `Overview` 一样，先显示当前聚焦健康项的优先失败项与首个受管阻塞摘要，再看次级健康项状态
- 当终端高度继续变低时，`Inspect` 会优先保住右侧 `Inspect Summary + Health Details`，并把 `Command Detail` 压成更短的辅助区；命令详情本身也只保留 `命令 / 状态 / 预览 / 分组 / 动作` 五行，预览会去掉 `nix --extra-experimental-features ...` 这类纯噪声前缀
- 只暴露归属 `Inspect` 的命令：
  - `flake check`
  - `check upstream pins`
- 支持 `j/k` 在 Inspect 动作间切换
- 支持 `x` 直接执行当前 Inspect 动作

这意味着：

- 健康状态和检查命令现在有了明确归宿
- `Overview -> Inspect` 已经形成固定路径

## 0.2 `Apply` 页

当前 `Deploy` 顶层页已经在语义上演进成 `Apply`。

当前行为：

- 左侧显示执行门槛、状态分类和命令预览
- 左侧 `Apply Summary` 固定先给 `当前判断 -> 最近结果 -> 下一步 -> 主动作`，再往下展示 `blocker / warning / handoff / info`
- `Apply Summary` 顶部四行现在也优先压成短句：像 `Apply 完成：switch nixos`、`回 Overview 复查`、`按 x 执行`、`进 Advanced` 会优先出现，避免默认主路径先被长反馈句打断
- `Apply Summary` 的四类分类项现在压成“首个关键信号 + 额外项计数”，像 `sudo -E 执行`、`远端固定版本 -> Advanced`、`hostname=nixos` 这种稳定短标签会优先出现，避免把整句状态文案原样塞进摘要框
- `Apply Preview` 现在统一用短标签 `目标 / 任务 / 来源 / 动作 / 升级 / 同步 / 执行`；同步预览会尽量压成 `source -> target`，直接执行命令会去掉冗余 `env`，交接或阻塞态也会收成稳定短句
- 当终端宽度继续变窄时，`Apply` 不再硬撑左右双栏；会优先把 `Apply Summary + Apply Preview` 放在上半区，再把 `Current Selection + Apply Controls` 下沉到下半区，确保默认主路径仍占第一视野
- 当终端高度继续变低时，`Apply` 会进一步扩大左侧主路径的宽度和预览高度，并压缩右侧 `Current Selection` 高度，优先保住 `Apply Summary + Apply Preview` 的可扫描性
- 右侧 `Current Selection` 在标准高度下仍是 4 行，但低高度时会按同一批信号压成 3 行或 2 行，不再为了塞下说明复制第二套状态文案
- 右侧现在明确是 `Apply Controls`，只保留当前 host 的默认 Apply 控件
- `Apply Controls` 在低高度或窄宽下会把已知长标签压成短标签，例如 `目标主机 -> 主机`、`区域切换 -> 区域`，`Enter 进入 Advanced -> Enter->Advanced`
- `Current Selection` 现在只保留 `建议 / 执行状态 / 当前聚焦 / Advanced 交接提示` 四类信息，不再重复 `默认目标` 和按键说明，避免右侧在短终端里变成第二个摘要块
- 区分 `blocker / warning / handoff / info`
- `repo-integrity` / `doctor` 的硬失败现在会直接进入 `Apply` blocker；blocker 优先级高于 handoff
- direct apply 路径下允许直接执行
- `Apply` 自身不再内嵌高级工作区；最后一行 `区域切换` 固定把用户送进顶层 `Advanced`
- 复杂来源时，交给 `Advanced` 承接完整 deploy wizard

这意味着：

- 日用路径已经从“先调参数，再决定要不要跑”转成“先看能不能 Apply，再处理高级项”
- 完整 deploy wizard 还保留，但角色已经变成高级路径

## 0.3 `catalog` 的当前定位

`catalog/` 现在不再应该被理解成“另一份软件搜索站点”。

当前主线里：

- `catalog/packages/*.toml` 保留仓库自己的包元数据
- `catalog/groups.toml` 保留 TUI 分组
- `catalog/workflows.toml` 承接项目自己的能力 / 工作流语义

也就是说，`catalog` 的价值不在于重复 nixpkgs 官网已有的包索引，而在于表达：

- 这个仓库为什么推荐某个软件
- 它属于哪条工作流
- 当前生命周期是 `stable / preview / legacy / deprecated` 中的哪一类

当前 `Packages` 页已经开始把这些项目自有元数据变成真实交互：不仅会显示 workflow / lifecycle，还支持按 workflow 过滤，并在左侧摘要里直接显示当前工作流的说明、可选数量和已选数量。以后继续扩展时，也应优先往这条“工作流与策略”主线走，而不是继续堆一份更大的静态包目录。

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
- `Packages` / `Home` / `Users` / `Hosts` 页会在右侧摘要里提前显示 `受管保护` 结果，不再等按下保存才暴露兄弟分片损坏或非受管陈旧组文件
- `Overview` / `Inspect` 也会汇总四条写回链的 `受管保护` 快照，方便在不切页的情况下先定位保存阻塞点
- `Overview` 的推荐主动作会把 `受管保护` 阻塞直接路由到对应页面，而不是只停留在总览提示
- 进入目标页面后，`Overview` 还会尽量把焦点落到与该阻塞最相关的字段、分组或设置项
- `doctor` 现在把缺少 `nixos-rebuild` 视为环境警告而不是仓库失败；真实部署路径仍会在执行前硬检查它

## 2. 受管文件保护

`mcbctl` 现在不会再无条件覆盖 `managed/*.nix`。

当前规则：

1. 新写入文件带 `mcbctl-managed` 标记和校验摘要
2. 旧占位文件或旧受管格式通过 `mcbctl migrate-managed` 显式迁移
3. 残留在 `managed/` 里的手写模块通过 `mcbctl extract-managed` 显式抽离到 `local.auto.nix`
4. 已带标记但内容被手改破坏的文件会被拒绝覆盖
5. `Home` / `Users` / `Hosts` 保存前，会连同同一 `managed` 子树里的兄弟分片一起核对 kind + checksum
6. `managed/packages/` 里混入非受管文件时，TUI 不会偷偷删掉它们
7. `repo-integrity` / `lint-repo` 会把旧格式、坏 checksum、错误 kind 和旧根目录硬件路径直接报错

这意味着：

- 受管分片是 Rust 独占写回区域
- 手写逻辑应搬到 `default.nix`、`packages.nix` 或 `local.nix`
- 单页保存不会绕过同目录下已经损坏的受管分片
- 保存失败会留在 TUI 内报告，而不是把整个会话打掉
- `local.auto.nix` 只是自动救援落点，不是长期主入口

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

`Apply` 页只在简单组合下直接执行；复杂来源或高级路径退回完整向导。

额外约束：

- `switch` / `test` / `boot` 现在要求 `hosts/<host>/hardware-configuration.nix` 存在
- `build-host` / `rebuild build` 可以只使用 `hosts/_support/hardware-configuration-eval.nix` 做评估

当前 wizard 语义：

- `Back` 回到上一个真实交互步骤，而不是上一个数字步骤
- 因此当 `per-user TUN` 关闭时，从 GPU 返回会直接回到管理员步骤，不会在 `step4 -> step5` 之间打转
- server host 从 server override 返回时，也会正确退回到 `per-user TUN` 或管理员步骤，而不是在 server override 内部循环
- server host 如果从 summary 返回 server override，再改成“沿用主机现有配置”，上一轮写入的 override 字段也会被清空
- 同步前和重建前各有一次确认；任一处拒绝都会中止主流程，但仍会进入 DNS / 临时目录收尾
- 如果主流程成功而收尾失败，最终错误会提升为 `部署收尾清理失败`，而不是静默吞掉 cleanup 问题
- 非交互模式会直接采用保守默认：
  - `ManageUsers` 使用当前仓库本地源
  - `UpdateExisting` 跟随远端分支 HEAD
  - 已有 host
  - 解析出的默认用户
  - 首个管理员用户
  - 桌面 GPU 自动识别结果
  - server override 关闭

当前探测读取语义：

- `missing`：允许继续按回退路径处理
- `unreadable`：会显式告警，再继续回退
- 这条规则当前已经落在默认用户来源解析、现有 `home/users/*` 枚举、GPU Bus ID 默认探测、host profile 判定、per-user TUN 默认探测、默认路由接口探测、当前 uid 探测上
- `per-user TUN` 优先尝试 `nix eval`；如果 `nix eval` 失败、输出不是 `true|false`，或候选文件不可读，会告警后退回文件扫描或默认 `false`
- GPU 自动识别优先尝试 `lspci -D`；如果 `lspci` 缺失则静默退回受管配置候选值，如果 `lspci` 执行失败则显式告警后回退
- 本地源和远端源提交都优先尝试 `git rev-parse HEAD`；如果命令失败或输出为空，会显式告警并继续后续复制/拉取，同时清空旧的 `source_commit`
- 备份时间戳优先尝试 `date +%Y%m%d-%H%M%S`；如果命令失败或输出为空，会显式告警并回退到 `unknown`
- 仓库自检里，如果存在 `mcbctl/Cargo.toml` 且系统有 `cargo`，会额外运行一次 `cargo check --quiet`；这条检查属于增强型门禁，命令失败会中止，但缺 `cargo` 或缺 `mcbctl/Cargo.toml` 只告警并跳过
- 如果最终仍无法识别 GPU 拓扑，TTY 问答会退回手动 GPU 方案；即使没有任何 Bus ID 候选，也必须允许手工输入 Intel / AMD / NVIDIA Bus ID
- 默认路由接口优先尝试 `ip route show default`；如果 `ip` 缺失则静默跳过，如果命令异常或输出无效则显式告警后回退
- 当前 uid 优先尝试 `id -u`；如果 `id` 缺失或探测异常，则显式告警并按非 root 环境继续
- `can_write_dir()` 只用于 rootless 的目录可写性探针；探针文件写入成功后的删除保持 best-effort，删除失败不会升级成部署收尾清理错误，也不会阻止后续 fallback 到 `$HOME/.nixos`
- `MCBCTL_COPY_USER_TEMPLATE` 只是一条可选脚手架开关；只有字面值 `true` 才复制模板用户目录内容，缺失、无效值或读取失败都等价于关闭

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
- 检查 `managed/*.nix` 是否仍符合 `mcbctl-managed` 协议
- 禁止继续使用仓库根目录 `hardware-configuration.nix`

`flake check` 只负责调用这些 Rust 检查器和 Rust 构建，不再在 Nix builder 里承载项目脚本逻辑。

## 6.1 `Actions` 的当前定位

独立的 `Actions` 叶子页已经删除。

当前不再保留单独的 `Actions` 状态模块；动作直接归各自页面持有：

- `FlakeCheck` / `UpdateUpstreamCheck` -> `Inspect`
- `FlakeUpdate` / `UpdateUpstreamPins` / `LaunchDeployWizard` -> `Advanced`
- `Apply` 的 sync/rebuild 只作为 `Apply Preview + Apply Current Host` 内部执行链存在，不再表示成独立 `ActionItem`

也就是说，`Actions` 不再以单独页面或单独状态模块继续存活；后续如果继续收口，应该直接在各自归宿区内处理动作，而不是重新恢复过渡页。

## 7. Release 资产

发布流程现在分两段：

1. `mcb-deploy release` 默认用当前 `mcbctl` 包版本创建 tag 和 GitHub Release
2. 它随后主动以这个 tag 触发 `.github/workflows/release-mcbctl.yml`

真正的资产布局由 Rust 子命令 `mcbctl release-bundle` 决定，而不是写死在 CI shell 里。当前 release 资产会按目标平台打成归档，并附带 `.sha256` 文件；`mcb-deploy release` 还会额外挂上一份 `release-manifest.json`，把 tag、release URL、各平台归档 URL 和 `.sha256` URL 收成统一机器可读入口，供 AUR / Homebrew / nixpkgs 等外部发布路径消费。

release 探测语义现在也和 deploy 主线保持一致：

- `git describe` 失败时，会显式告警并按“首次发布”生成 notes
- `git log` 失败时，会显式告警并生成回退版 release notes
- `git status --porcelain` 探测失败时，会直接中止 release，避免把未知工作区状态误判成 clean

这意味着：

- release 版本不再默认走日期 tag 递增，而是要求显式版本管理
- CI 资产会和 release tag 对齐，不再依赖可继续前进的分支 head
