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
Overview -> Preview Apply -> Apply Current Host
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
- 每个现有 `ActionItem` 必须有唯一归宿
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
| 阻塞项 `blocker` | 会禁止 `Apply Current Host` 直接执行的条件 | `execute_deploy()` 前置检查和 `Actions` 守卫 |
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

`Overview` 只能有这三个一等动作：

- `Preview Apply`
- `Apply Current Host`
- `Save Dirty Pages`

次级动作只能放在第二层：

- `Open Edit`
- `Open Advanced`
- `Open Inspect`

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

## 5. `Apply` 规格

`Apply` 是当前 `Deploy` 的替代，不是另一套部署模型。

它只负责“当前 host 的默认安全路径”，不直接取代完整 wizard。

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
| area switch | `Page::Advanced` / `show_advanced` | `Enter` 进入顶层 `Advanced`，或从兼容区返回 `Apply` |
| sync preview | `deploy_sync_plan_for_execution()` | 是否需要把 repo 同步到 `/etc/nixos` |
| rebuild preview | `deploy_rebuild_plan_for_execution()` | 预览实际 `nixos-rebuild` 调用 |

`show_advanced` 只表示 Apply 内兼容高级工作区已打开；顶层 `Advanced` 由 `Page::Advanced` 单独承载。该 flag 打开时必须 handoff，不允许 direct apply。

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
| `show_advanced == true` | `can_execute_deploy_directly()` | `handoff` | `Open Advanced` | 仅表示 Apply 内兼容高级工作区已打开；顶层 `Advanced` 本身不再依赖这个 flag |

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

`Apply` 页只保留这 4 个固定按钮：

- `Run Preview`
- `Apply Current Host`
- `Open Advanced`
- `Back`

按钮规则固定为：

| 按钮 | 可见性 | 何时禁用 |
|---|---|---|
| `Run Preview` | 总是可见 | 仅在 `target_host` 为空时禁用 |
| `Apply Current Host` | 仅在 direct apply 路径下可见 | 任一 `block` 存在时禁用 |
| `Open Advanced` | 总是可见 | 不禁用 |
| `Back` | 总是可见 | 不禁用 |

## 6. `Actions` 拆分规格

`Actions` 不是“以后也保留，但顺便多几个入口”，而是明确要拆掉。

### 6.1 唯一映射表

| 当前 `ActionItem` | 未来区域 | 未来分组 | 是否保留直接执行 | 说明 |
|---|---|---|---:|---|
| `FlakeCheck` | `Inspect` | Repo Checks | 是 | 纯检查 |
| `FlakeUpdate` | `Advanced` | Repository Maintenance | 是 | 变更仓库，不应和 Inspect 混在一起 |
| `UpdateUpstreamCheck` | `Inspect` | Upstream Pins | 是 | 纯检查 |
| `UpdateUpstreamPins` | `Advanced` | Repository Maintenance | 是 | 会回写 source.nix |
| `SyncRepoToEtc` | `Apply` | Manual Apply Helpers | 是 | 与当前 host 应用路径相关 |
| `RebuildCurrentHost` | `Apply` | Manual Apply Helpers | 是 | 与当前 host 应用路径相关 |
| `LaunchDeployWizard` | `Advanced` | Deploy | 是 | 完整高级路径 |

### 6.2 迁移后的长期规则

- `Inspect` 只放“读取 / 检查 / 诊断类”动作
- `Apply` 只放“当前 host 的应用相关”动作
- `Advanced` 放“会改变仓库状态、需要复杂参数、或需要完整 wizard 的动作”

不允许把 `FlakeUpdate`、`UpdateUpstreamPins` 重新塞回 `Inspect`。

### 6.3 `Actions` 页的收尾方式

`Actions` 页在迁移期最多只允许保留一种形态：

- 作为过渡入口页，列表项直接跳转到 `Apply / Advanced / Inspect`

不允许继续维持“在过渡页里直接执行所有动作”的最终形态。

## 7. 现有页面到新结构的硬映射

| 现有页 | 新结构角色 | 是否继续作为独立页存在 | 备注 |
|---|---|---:|---|
| `Dashboard` | `Overview` | 是 | 直接替换，不双轨并存 |
| `Deploy` | `Apply` | 是 | 直接替换，不再保留旧 deploy 文案 |
| `Packages` | `Edit` | 是 | 保持 |
| `Home` | `Edit` | 是 | 保持 |
| `Users` | `Edit` | 是 | 保持 |
| `Hosts` | `Edit` | 是 | 保持 |
| `Actions` | 过渡入口，最终拆除 | 否 | 不作为长期顶层页 |

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
2. `Apply Current Host`
3. `Save Dirty Pages`

`Open Advanced` 不能和 `Apply Current Host` 抢同一层级的视觉权重。

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
