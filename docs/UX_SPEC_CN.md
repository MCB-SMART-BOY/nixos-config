# `mcbctl` 主线 UX 规格

这份文档不是方向说明，而是去二义性的实现规格。

如果它和 [UX_MAINLINE_CN.md](./UX_MAINLINE_CN.md) 冲突，以这份文档为准。

## 1. 适用范围

这份规格只约束 `mcbctl` TUI 主线，不改变这些既有边界：

- `Packages / Home / Users / Hosts` 的受管写回边界不变
- 完整 `mcb-deploy` wizard 继续保留
- `mcbctl` / `mcb-deploy` / `flake packages/apps` 的入口不变
- 底层执行链仍以当前 Rust 实现为准，不重新引入 shell 主线

它要解决的是“默认日用入口该怎么组织”，不是“底层逻辑要不要改写”。

## 2. 去二义性的顶层决定

下面这些决定在后续实现里必须固定，不允许同一时期并存两套主线。

### 2.1 顶层区域

未来的顶层信息架构固定为：

- `Overview`
- `Edit`
- `Apply`
- `Advanced`
- `Inspect`

### 2.2 现有页面和新区域的关系

- `Dashboard` 不是保留页，它会被直接演进成 `Overview`
- `Deploy` 不是保留原样，它会被直接演进成 `Apply`
- `Actions` 不是长期保留页，它会被拆到 `Apply / Advanced / Inspect`
- `Packages / Home / Users / Hosts` 继续作为 `Edit` 区里的具体编辑页

### 2.3 默认主路径

默认主路径固定为：

```text
Overview -> Preview Apply -> Apply Current Host -> Result
```

默认主路径不允许先要求用户选择：

- source strategy 细项
- 远端 ref
- 新 host / 新用户脚手架
- GPU Bus ID
- per-user TUN 明细
- server override
- release 参数

这些全部后移到 `Advanced`。

### 2.4 `Actions` 的命运

`Actions` 是过渡页，不是长期主结构。

最终要求：

- 不再保留“一个页里同时放 inspect、apply、advanced 三类动作”的结构
- 每个仍保留的页面动作必须有唯一归宿
- 不允许同一动作在 `Actions` 和新区域里长期双轨暴露

## 3. 术语和真值来源

这几个词在后续实现中必须只有一个含义。

| 术语 | 定义 | 当前真值来源 |
|---|---|---|
| 当前主机 `current host` | 运行 TUI 的宿主机名 | `AppContext.current_host` |
| 目标主机 `target host` | 当前 Apply/Deploy 默认要作用到的 host | `AppState.target_host` |
| 当前仓库 `repo root` | 当前 TUI 绑定的 flake 根 | `AppContext.repo_root` |
| `/etc/nixos` 根 | 当前机器系统配置根目录 | `AppContext.etc_root` |
| 当前权限模式 | `root / sudo-session / sudo-available / rootless` | `AppContext.privilege_mode` |
| dirty | 某一页存在尚未保存到受管文件的内存改动 | 四个 dirty 集合 |
| 阻塞项 `blocker` | 会禁止 `Apply Current Host` 直接执行的条件 | `execute_deploy()` 前置检查和页面自有状态守卫 |
| 警告项 `warning` | 不阻止执行，但必须在预览里明确告知的条件 | 预览模型派生 |
| 交接项 `handoff` | 不报错，但当前页必须交给 `Advanced` 完成的条件 | `can_execute_deploy_directly()` 和 wizard 路由 |

### 3.1 dirty 的唯一来源

`dirty` 只能来自下面四组状态：

- `host_dirty_user_hosts`
- `host_dirty_runtime_hosts`
- `package_dirty_users`
- `home_dirty_users`

任何新的首页/Overview/Apply 状态摘要，都只能从这四组派生；不允许再引入第二套“未保存状态”判断。

### 3.2 目标主机默认值

`target_host` 的默认决策固定为：

1. 如果 `current_host` 在 `hosts/` 里存在，就用它
2. 否则如果存在 `nixos`，就用 `nixos`
3. 否则取第一个可用 host
4. 如果 `hosts/` 为空，才退回 `current_host`

这个规则来自当前 `default_target_host()`，后续首页和 Apply 页都必须复用，不允许各页各自猜。

## 4. `Overview` 规格

`Overview` 是当前 `Dashboard` 的替代，不是额外新增的第六层。

### 4.1 `Overview` 的目标

用户进入 `Overview` 后，必须在 5 秒内回答这 5 个问题：

1. 当前默认要操作哪台 host
2. 仓库结构是否健康
3. 当前环境是否允许执行 Apply
4. 哪些页面有未保存改动
5. 如果现在不能 Apply，卡在哪

### 4.2 `Overview` 必须显示的字段

| 区块 | 字段 | 真值来源 | 刷新规则 | 是否影响 `Apply` |
|---|---|---|---|---|
| Current Context | 当前主机 | `context.current_host` | 应用启动、手动刷新 | 否 |
| Current Context | 目标主机 | `target_host` | 进入 TUI 后即时更新 | 是 |
| Current Context | 当前用户 | `context.current_user` | 应用启动、手动刷新 | 否 |
| Current Context | 权限模式 | `context.privilege_mode` | 应用启动、手动刷新 | 是 |
| Current Context | 当前仓库 | `context.repo_root` | 应用启动、手动刷新 | 是 |
| Current Context | `/etc/nixos` | `context.etc_root` | 应用启动、手动刷新 | 是 |
| Apply Snapshot | 默认来源 | `deploy_source` | Apply 模型变化时即时更新 | 是 |
| Apply Snapshot | 默认动作 | `deploy_action` | Apply 模型变化时即时更新 | 是 |
| Apply Snapshot | 是否将同步 `/etc/nixos` | `deploy_sync_plan_for_execution().is_some()` | Apply 模型变化时即时更新 | 是 |
| Dirty | Users dirty | `host_dirty_user_hosts` | 编辑/保存后即时更新 | 是 |
| Dirty | Hosts dirty | `host_dirty_runtime_hosts` | 编辑/保存后即时更新 | 是 |
| Dirty | Packages dirty | `package_dirty_users` | 编辑/保存后即时更新 | 是 |
| Dirty | Home dirty | `home_dirty_users` | 编辑/保存后即时更新 | 是 |
| Health | Host 配置可用性 | `host_settings_by_name + host_settings_errors_by_name` | 切换 target host、刷新 host 配置后更新 | 是 |
| Health | Host 配置校验结果 | `ensure_host_configuration_is_valid(target_host)` | target host 变化、host 状态变化后更新 | 是 |
| Health | Repo integrity | `repo-integrity` 快照 | 启动后首次计算；显式刷新时重算 | 是 |
| Health | Doctor | `doctor` 快照 | 不自动后台刷新；显式刷新时重算 | 是 |

### 4.3 `Overview` 刷新规则

为了避免 UI 抖动和状态歧义，`Overview` 的刷新规则固定为两类：

- 即时派生
  - `target_host`
  - `deploy_source`
  - `deploy_action`
  - dirty 汇总
  - host 配置可用性和校验结果
- 显式刷新
  - `repo-integrity`
  - `doctor`
  - 任何需要实际调用外部命令的健康检查

`Overview` v1 不做后台自动轮询。

当前 `Overview` v1 的健康项行为固定为：

- `repo-integrity` 在 TUI 启动时先计算一次并缓存
- `doctor` 初始为 `NotRun`，只在显式刷新后更新
- `r`：只刷新 `repo-integrity`
- `d`：只刷新 `doctor`
- `R`：顺序刷新两者

这三组键位只更新 `Overview` 健康缓存和状态提示，不额外改变 `Apply` 配置字段。

### 4.4 `Overview` 主动作

`Overview` 在 M2 收口阶段只能保留一个一等动作：

- `Preview Apply`

从 `Overview` 进入 `Apply` 时，右侧控件焦点必须重置到第一行 `目标主机`。

不允许沿用历史 `区域切换` / `Advanced` 焦点，把用户一落地就带去专家路径。

下面这些能力只能作为次级入口保留，不能抢主动作：

- `Open Edit`
- `Open Advanced`
- `Open Inspect`
- `Save Dirty Pages`

`Overview` 左上摘要在 M2 也必须固定阅读顺序：

1. `当前判断`
2. `原因`
3. `最近结果`
4. `下一步`
5. `主动作`

`Overview` 这个摘要块标题固定为 `Overview Summary`。

其中：

- `原因` 只是解释为什么当前主动作成立
- 不允许把 `主动作` 再放回摘要最上方，打断默认扫描顺序
- 这组顺序必须和 `Apply / Inspect` 的摘要壳保持同一阅读节奏
- `Apply Snapshot` 只保留 `默认来源 / 默认动作 / 同步` 这类配置快照，不允许重复 `Apply Summary` 已经给出的 `blocker / warning / handoff / 当前判断`
- `Health` 必须先给一行稳定的 `host-config` 摘要，再展示当前聚焦健康项和 `save-guards`
- `Dirty State` 只保留 `状态 / 待保存 / 优先保存` 三类信息，不再混入额外的 `Apply 影响` 解释句
- `Current Context` 应优先压成成对字段行：`目标/当前`、`用户/权限`，再展示仓库路径，避免短终端下摘要区失衡

### 4.5 `Save Dirty Pages` 的 v1 语义

这里必须先固定边界，避免实现时理解分叉。

`Save Dirty Pages` 在 v1 的语义是：

- 只保存当前内存里已经 dirty 的页
- 不做任何新的探测和自动补全
- 任一页保存失败，整个批次立即停止
- 成功保存的页不回滚
- 失败信息必须明确指向失败页和失败对象

`Save Dirty Pages` 不负责：

- 自动切换 host / user
- 自动修复 blocker
- 自动打开 Advanced

## 4.6 `Inspect` 壳补充

`Inspect Summary` 的默认阅读顺序必须和 `Overview / Apply` 一致：

1. `当前判断`
2. `最近结果`
3. `下一步`
4. `主动作`

这个摘要块标题固定为 `Inspect Summary`。

其中：

- 顶部 4 行在默认表面必须优先压成稳定短句，不允许把“先看详情；如需复查，再按 x ...”这类长句原样塞满摘要块
- `Health Details` 必须始终位于 `Command Detail` 之前；低高度时应优先保住 `Inspect Summary + Health Details`，而不是优先保住命令说明区
- `Command Detail` 只保留 `命令 / 状态 / 预览 / 分组 / 动作` 五类信息，不允许继续混入第二套“最近结果”或长操作说明
- `Command Detail` 的 `预览` 应优先去掉纯噪声前缀，例如 `nix --extra-experimental-features 'nix-command flakes'`
- `Inspect Commands` 在窄宽下允许把分组压成短标签，例如 `Repo Checks -> Repo`、`Upstream Pins -> Pins`，但动作归宿和可执行性真值来源不变

## 4.7 `Edit` 壳补充

`Edit` 在 M2 收口阶段不能只靠 footer 让用户记忆自己当前在哪个子页。

`Edit Pages` 顶栏必须直接显示 4 个固定子页：

- `Packages`
- `Home`
- `Users`
- `Hosts`

并且：

- dirty 子页必须直接在 tab 标题上标记，例如 `Packages*`
- 这个 dirty 标记只能复用现有 4 个 dirty 集合，不允许引入第二套未保存判断
- `Edit Workspace` 固定只保留 3 行：`当前页/目标`、`Dirty`、`建议`
- `建议` 必须优先指出当前页 dirty，其次指出当前页受管保护，再回退到其他 dirty/guard 页
- `Home` / `Users` / `Hosts` 的叶子标题必须使用目标导向短标题，例如 `Home (alice)`、`Users (demo)`、`Hosts (demo)`；不允许继续保留 `Home Settings`、`Users Model`、`Host Override` 这类历史标题
- `Packages` 摘要标题固定为 `Packages Summary`；`Home` 摘要标题固定为 `Home Summary`
- `Edit` 正文在窄宽下必须优先保住主列表与当前页定位：
  - `Home / Users / Hosts` 改为上方主列表、下方摘要
  - `Packages` 改为上方 `Packages Summary`、下方 `Packages 列表 + Selection`
  - 更窄时 `Packages Summary / Packages 列表 / Selection` 继续顺序堆叠
- `Home Summary` 在窄宽或低高下必须把长状态和长说明压成稳定短句；默认优先保留 `用户 / 目标 / 聚焦 / 状态 / 受管保护 / 写回`
- `Packages Summary` 在窄宽或低高下不得继续逐条原样输出过滤、搜索、统计、工作流和落点字段；必须优先压成 `源/用户 / 过滤 / 数量 / 当前流程 / 当前组 / 状态 / 受管保护` 这类稳定短行；目标目录和当前组落点只允许在非 tight 模式保留
- `Home / Users / Hosts` 的摘要必须能够在当前页 scoped feedback 命中时显示 `最近结果 / 下一步`；这些结果只能复用现有 `UiFeedbackScope::Home / Users / Hosts`，不允许再引入第二套页内结果状态
- `Packages` 列表在窄宽下必须把长标题收成短标题 `Packages`；列表项允许压成 `[x] name [category/group]`，更窄时允许只保留 `[x] name`
- `Selection` 在窄宽或低高下不得继续原样输出完整长段落；必须优先保留 `条目 / 类组 / 组 / 流程 / 已选 / 状态` 这类稳定摘要；工作流差异和最近结果允许压成单行摘要，但不允许引入第二套状态真值
- `Packages Selection.status` 必须是稳定页内状态，不允许直接复用全局 legacy `status`；最近一次过滤、搜索、分组、workflow 和保存反馈必须单独走 `UiFeedbackScope::Packages` 派生的 `最近结果 / 下一步`
- `Edit` 的默认 footer 必须先复用一套共同骨架：`Edit/<Page> | 1-4 子页 | ←/→ 目标 | j/k 移动 | ... | ? 帮助`；最后一个 `...` 才允许替换成当前页主动作
- `Edit` 的 `?` 帮助面板必须固定为：先看 `Edit Workspace + 当前页主列表/摘要`，再看共同骨架，最后看当前页主动作和扩展键；不允许继续让四个编辑页各自使用不同章节结构
- 这些布局变化只允许发生在现有 4 个编辑页壳层，不允许引入新的编辑页、wizard 或第二套摘要真值

## 5. `Apply` 规格

`Apply` 是当前 `Deploy` 的替代，不是另一套部署模型。

它只负责“当前 host 的默认安全路径”，不直接取代完整 wizard。

`Apply` 左侧摘要在 M2 必须固定为：

1. `当前判断`
2. `最近结果`
3. `下一步`
4. `主动作`
5. `blocker`
6. `warning`
7. `handoff`
8. `info`

这个摘要块标题固定为 `Apply Summary`。

也就是说：

- 先让用户知道当前能不能走默认主路径
- 再给分类后的执行门槛
- 不允许把分类项提前到摘要壳前面，抢掉默认决策顺序
- 顶部 `当前判断 / 最近结果 / 下一步 / 主动作` 在默认表面也必须优先压成稳定短句；不允许把路由反馈、成功反馈或长动作说明原样塞满摘要框
- `blocker / warning / handoff / info` 在默认表面必须优先压成稳定短标签；如果同类有多项，只显示首项并追加 `另 N 项`，不允许把多句长文案原样堆进摘要框

### 5.1 `Apply` 页必须暴露的模型

`Apply` v1 必须始终可见这 9 项：

| 字段 | 真值来源 | 说明 |
|---|---|---|
| target host | `target_host` | 当前默认作用对象 |
| task | `deploy_task` | 只作说明，不改变 `Apply Current Host` 的 blocker 规则 |
| source | `deploy_source` | 决定 direct apply 还是 handoff |
| pinned ref | `deploy_source_ref` | 仅当 `source == RemotePinned` 时可编辑；为空时禁止启动 deploy wizard |
| action | `deploy_action` | `switch / test / boot / build` |
| flake update | `flake_update` | 是否在 rebuild 前执行 upgrade |
| area switch | `Page::Advanced` | `Enter` 进入顶层 `Advanced` |
| sync preview | `deploy_sync_plan_for_execution()` | 是否需要把 repo 同步到 `/etc/nixos` |
| rebuild preview | `deploy_rebuild_plan_for_execution()` | 预览实际 `nixos-rebuild` 调用 |

`Apply` 不再维护内部高级工作区状态；最后一行 `区域切换` 只负责把用户送进顶层 `Advanced`。`Advanced` 由 `Page::Advanced` 单独承载，direct apply 资格只由当前来源决定。

`Apply Preview` 在默认表面应优先使用短标签：

- `目标`
- `任务`
- `来源`
- `来源细节`
- `动作`
- `升级`
- `同步`
- `执行`

其中：

- `同步` 优先压成 `source -> target`，而不是整行 `rsync` 命令
- `执行` 在 direct apply 路径下应去掉纯噪声前缀，例如冗余的 `env`
- 当当前页不会直接执行时，`执行` 应改成稳定短句，例如 `交给 Advanced：...` 或 `先处理 blocker / warning`

当宽度不足以同时承载两栏且不压垮左侧主路径时，`Apply` 必须切到左侧优先的堆叠布局：

- 上半区：`Apply Summary` + `Apply Preview`
- 下半区：`Current Selection` + `Apply Controls`

不允许为了保住双栏外观，继续让右侧说明把左侧默认主路径压到不可扫描。

当高度继续变低时，`Apply` 也必须优先保住左侧默认主路径：

- 左侧列应获得更多宽度，优先减少 `Apply Summary` 与 `Apply Preview` 的换行
- `Current Selection` 高度应先于左侧主路径被压缩
- 在窄宽且低高度同时出现时，应继续优先扩大上半区 `Apply Summary + Apply Preview`，不允许把有限高度优先分给次级说明区

`Current Selection` 在默认表面只保留这 4 类稳定信号：

- `建议`
- `执行状态`
- `当前聚焦`
- `Advanced` 交接提示

在标准高度下，这 4 类信号可以各占 1 行。

当右侧高度继续被压缩时，允许按同一真值来源合并成更短表达：

- `Compact`：压成 3 行，`执行状态 + 当前聚焦` 合并
- `Tight`：压成 2 行，`建议 + 执行状态`、`当前聚焦 + Advanced` 分别合并

不允许为了压缩高度重新引入第二套 recommendation / handoff / blocker 文案来源；只能对当前已有信号做短标签化和合并显示。

不允许继续在这个区块重复 `默认目标`、长按键说明或第二套“下一步”文案；这些信息已经分别由左侧摘要、控制列表和 `?` 帮助面板承接。

当前 `Advanced Wizard` 还额外维护一组独立于 `Apply` 的 handoff snapshot：
- `advanced_target_host`
- `advanced_deploy_task`
- `advanced_deploy_source`
- `advanced_deploy_source_ref`
- `advanced_deploy_action`
- `advanced_flake_update`

这组值进入 `mcb-deploy` 时必须序列化成内部参数，不允许再回退到读取 `Apply` 页的同名字段。

### 5.2 `Apply` 的状态分类

`Apply` 页只能把条件分成 4 类：

- `block`
- `warning`
- `handoff`
- `info`

不允许再出现第五种“似乎能执行但其实会在命令里炸”的隐含状态。

### 5.3 `Apply` 阻塞矩阵

下面这些条件必须直接阻止 `Apply Current Host`：

| 条件 | 当前真值来源 | 分类 | `Apply` 按钮 | 必须给出的下一步 |
|---|---|---|---|---|
| 任一 dirty 集合非空 | `ensure_no_unsaved_changes_for_execution()` | `block` | 禁用 | `Save Dirty Pages` 或进入对应编辑页 |
| `repo-integrity` 失败 | `ensure_repository_integrity(repo_root)` | `block` | 禁用 | 打开 `Inspect` 查看失败项 |
| `target_host` 配置读取失败 | `host_settings_errors_by_name[target_host]` | `block` | 禁用 | 打开 `Hosts` 或 `Users` 修正 |
| `target_host` 无可用配置 | `host_settings_by_name[target_host]` 缺失 | `block` | 禁用 | 打开 `Advanced` 或补 host 配置 |
| `target_host` 校验失败 | `ensure_host_configuration_is_valid(target_host)` | `block` | 禁用 | 打开 `Users/Hosts` 修正 |
| `rootless` 且 `deploy_action != Build` | `execute_deploy()` 当前规则 | `block` | 禁用 | 切换到 root/sudo，或改成 `build` |
| `doctor` 的硬失败 | `doctor` 快照 | `block` | 禁用 | 修复宿主环境能力 |

### 5.4 `Apply` 交接矩阵

下面这些条件不是错误，但必须交给 `Advanced`：

| 条件 | 当前真值来源 | 分类 | 主按钮文案 | 说明 |
|---|---|---|---|---|
| `deploy_source == RemotePinned` | `can_execute_deploy_directly()` | `handoff` | `Open Advanced` | 远端固定版本不走 direct apply |
| `deploy_source == RemoteHead` | `can_execute_deploy_directly()` | `handoff` | `Open Advanced` | 远端最新版本不走 direct apply |

`handoff` 不应显示为失败，也不应显示为 warning。

### 5.5 `Apply` 警告矩阵

下面这些条件不阻止执行，但必须在预览里显式展示：

| 条件 | 当前真值来源 | 分类 | 必须展示的预览信息 |
|---|---|---|---|
| 当前仓库不等于 `/etc/nixos` 且将发生同步 | `deploy_sync_plan_for_execution().is_some()` | `warning` | 明确列出 `repo_root -> etc_root` 的同步路径 |
| `flake_update == true` | `flake_update` | `warning` | 明确说明会以 `--upgrade` 运行 |
| 当前环境将使用 `sudo` | `should_use_sudo()` | `warning` | 明确说明最终命令前缀会带 `sudo -E` |
| 当前页 direct apply 需要真实 hardware config | `deploy_action != Build` 或非 `rootless build` | `warning` | 明确说明会检查或要求 `hosts/<host>/hardware-configuration.nix` |

### 5.6 hardware config 的去二义性决定

这里必须单独写死，因为它直接影响用户信任。

`Apply` v1 的产品语义应该是：

- 预览阶段必须明确告诉用户这次执行是否要求真实 hardware config
- 如果缺失真实 hardware config，不允许把这件事藏成“执行时才发现”
- 允许保留当前底层的自动生成能力
- 但 UI 层必须把它表现成“显式步骤”，而不是无提示副作用

也就是说，未来实现时即使继续复用 `ensure_host_hardware_config()`，`Apply` 预览也必须先把这件事显示出来。

### 5.7 `Apply` 页按钮语义

`Apply` 页只保留这 2 类固定动作：

- `Apply Current Host`
- `区域切换 -> Enter 进入 Advanced`

动作规则固定为：

| 动作 | 可见性 | 何时禁用 |
|---|---|---|
| `Apply Current Host` | 总是作为默认主动作存在 | 任一 `block` 存在，或当前状态属于 `handoff` 时不可执行 |
| `Enter 进入 Advanced` | 总是可见 | 不禁用 |

如果同一时刻同时存在 `block` 和 `handoff`，UI 必须优先表现成 `block`，不能继续把默认下一步写成 `Open Advanced`。

## 6. `Actions` 拆分规格

`Actions` 不是“以后也保留，但顺便多几个入口”，而是明确要拆掉。

### 6.1 唯一映射表

| 当前 `ActionItem` | 未来区域 | 未来分组 | 是否保留直接执行 | 说明 |
|---|---|---|---:|---|
| `FlakeCheck` | `Inspect` | Repo Checks | 是 | 纯检查 |
| `FlakeUpdate` | `Advanced` | Repository Maintenance | 是 | 变更仓库，不应和 Inspect 混在一起 |
| `UpdateUpstreamCheck` | `Inspect` | Upstream Pins | 是 | 纯检查 |
| `UpdateUpstreamPins` | `Advanced` | Repository Maintenance | 是 | 会回写 source.nix |
| `LaunchDeployWizard` | `Advanced` | Deploy | 是 | 完整高级路径 |

### 6.2 迁移后的长期规则

- `Inspect` 只放“读取 / 检查 / 诊断类”动作
- `Apply` 不再通过独立 `ActionItem` 暴露 sync/rebuild；这些能力必须收口为 `Apply Preview + Apply Current Host` 内部执行链
- `Advanced` 放“会改变仓库状态、需要复杂参数、或需要完整 wizard 的动作”

不允许把 `FlakeUpdate`、`UpdateUpstreamPins` 重新塞回 `Inspect`。

### 6.3 `Actions` 页的收尾方式

当前实现应继续保持：

- 不再渲染独立的 `Actions` 叶子页
- `Inspect` 与 `Advanced` 各自持有页面动作；`Apply` 只保留默认执行路径，不再维护独立 helper 动作枚举

不允许为了兼容旧路径，再把 `Actions` 恢复成一个可见的过渡页。

## 7. 现有页面到新结构的硬映射

| 现有页 | 新结构角色 | 是否继续作为独立页存在 | 备注 |
|---|---|---:|---|
| `Dashboard` | `Overview` | 是 | 直接替换，不双轨并存 |
| `Deploy` | `Apply` | 是 | 直接替换，不再保留旧 deploy 文案 |
| `Packages` | `Edit` | 是 | 保持 |
| `Home` | `Edit` | 是 | 保持 |
| `Users` | `Edit` | 是 | 保持 |
| `Hosts` | `Edit` | 是 | 保持 |
| `Actions` | 历史过渡入口，已拆除 | 否 | 不作为长期顶层页 |

## 8. 文案和交互的硬规则

### 8.1 首页文案

首页优先使用任务词，不优先使用实现词。

允许出现在第一层的词：

- 当前主机
- 当前仓库
- 未保存
- 预览应用
- 应用到当前主机
- 高级部署
- 检查与诊断

不应在第一层直接暴露为主按钮的词：

- `source_ref`
- `specialisation`
- `override`
- `bus id`
- `per-user tun table`

### 8.2 主动作优先级

主动作优先级固定为：

1. `Preview Apply`

`Apply Current Host` 只存在于 `Apply` 页的执行位；`Open Advanced` 不能和它抢同一层级的视觉权重。

### 8.3 帮助系统

帮助系统在 v1 必须固定为：

- 默认页脚只显示当前页最常用的一小组快捷键
- 详细键位说明统一放进 `?` 帮助面板
- `?` 只在非文本输入模式下打开或关闭帮助面板
- 帮助面板打开时，不允许底层页面继续响应普通页面动作
- `Esc` 在非文本输入模式下只关闭帮助面板；在文本输入模式下继续表示取消当前输入

不允许继续把整页长键位说明塞回 footer。

## 9. v1 实施顺序

为了避免再次产生双轨结构，v1 的实施顺序固定为：

1. 先把 `Dashboard` 改成 `Overview`，但不改底层执行链
2. 再把 `Deploy` 改造成 `Apply`，以当前 direct apply 规则为准
3. 再把 `Actions` 做唯一映射拆分
4. 最后才处理顶层导航和交互细节收口

不允许先做一个新的 `Overview`，同时保留旧 `Dashboard` 长期并存。

## 10. 验收标准

这份规格要达成的最低验收标准是：

- 用户进入首页后 5 秒内能判断“能不能直接 Apply”
- `Apply` 页能明确区分 `block / warning / handoff / info`
- `Actions` 的每个现有动作都有唯一归宿
- 顶层不再出现 `Dashboard + Overview` 或 `Deploy + Apply` 的长期双轨
- `Preview Apply` 能显示同步、重建、权限、hardware config 要求
