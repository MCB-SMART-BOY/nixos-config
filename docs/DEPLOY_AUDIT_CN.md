# 部署主线审计基线

这份文档不是历史说明，而是接下来继续收 `mcb-deploy` / `mcbctl deploy` 主线时的工作基线。

如果要从“工程主线”继续往“日用体验主线”推进，产品级方案见 [UX_MAINLINE_CN.md](./UX_MAINLINE_CN.md)，去二义性的实现规格见 [UX_SPEC_CN.md](./UX_SPEC_CN.md)。

目标只有三个：

1. 让真实交互行为和内部状态机完全一致
2. 让剩余命令探测 / 外部命令调用的失败语义完全一致
3. 让后续继续补测试时，不需要再反复重新判断优先级

## 1. 风险分层

接下来所有工作按这三个等级推进：

- `P0`
  会导致错误状态继续向下执行、静默误配置、误导性成功，或让用户看到的交互与真实状态不一致
- `P1`
  不会立刻写坏配置，但会让诊断成本高、fallback 语义不一致、同类错误处理风格漂移
- `P2`
  主要是测试样板、文档漂移、helper 重复，不会直接造成业务事故

当前剩余主线工作里，`P0` 主要集中在真实 TTY 交互；`P1` 主要集中在剩余命令探测和工具层；`P2` 主要集中在测试夹具和文档入口。

## 2. TTY 交互场景矩阵

### 2.1 来源选择

- 场景：TTY 下选择本地源 / 远端 HEAD / 远端固定 ref
  当前状态：非 TTY 默认、prepare failure 后 `retry / reselect / exit`、非法输入重试、固定 ref 留空重试 已有测试；并已补一条真实 stdin/stdout 样例覆盖“非法菜单 -> 固定 ref -> 空值重试”
  剩余风险：如果后续还要继续收，只剩本地源默认和远端 HEAD 之类的输出样例
  下一步：把真实终端样例扩到别的高收益交互页时再补

- 场景：来源准备失败后重新选择策略
  当前状态：流程级测试、`1/2/3/q` 动作映射、以及“告警 + 下一步菜单”真实 stdin/stdout 样例都已覆盖
  剩余风险：如果后续还要继续收，只剩更长链路的端到端终端样例
  下一步：把真实终端样例转到别的尚未覆盖场景

### 2.2 Host 选择

- 场景：已有 host 选择、新建 host、`UpdateExisting` 禁止新建
  当前状态：默认 host、保留目录过滤、TTY 非法输入重试、新建 host 名称重试 已覆盖；并已补真实 stdin/stdout 样例覆盖“已有 host 菜单重试”和“新建 host 名称校验”；同时已补流程级回归覆盖“桌面 -> 服务器 / 服务器 -> 桌面”的 host 重选后再次进入 users/admin/runtime，确认旧 host 的 profile / GPU / TUN / server override 状态会被清空；并已补一条真实向导 transcript 覆盖“host -> users/admin/runtime -> 返回 -> host -> 再进入 runtime”
  剩余风险：Host 选择本身已基本收口，剩余主要是更大的跨页面端到端 transcript，而不是 host 链路语义空洞
  下一步：转阶段收口审查，确认这一批 transcript 与门禁补丁是否达到发版候选标准

### 2.3 Users / Admin

- 场景：选择已有用户、新增用户、清空用户、返回上一页
  当前状态：非 TTY 默认、非法用户名重试、清空后禁止直接完成 已覆盖；并已补一条真实 stdin/stdout 样例覆盖“清空后直接完成 -> 警告”和“非法用户名 -> 重试”
  剩余风险：如果后续还要继续收，只剩 users/admin 往返后的更长链路终端样例
  下一步：把真实终端样例转到别的交互页

- 场景：管理员选择
  当前状态：默认管理员、清空后禁止直接完成 已覆盖；并已补一条真实 stdin/stdout 样例覆盖“清空管理员 -> 直接完成 -> 警告”
  剩余风险：如果后续还要继续收，只剩跨步骤回退后的更长链路终端样例
  下一步：把真实终端样例转到别的交互页

### 2.4 per-user TUN

- 场景：开启 / 关闭、为每个用户填写接口和 DNS 端口
  当前状态：状态机回退已覆盖；并已补真实 stdin/stdout 样例覆盖“空输入走默认值 + 预览确认”和“非法端口触发整轮重输”
  剩余风险：如果后续还要继续收，只剩用户列表变化、返回上一步之后的更长链路终端样例
  下一步：把真实终端样例转到别的交互页

### 2.5 GPU

- 场景：自动检测、手工模式、Bus ID 选择 / 手填、specialisation 选择
  当前状态：非 TTY 默认、关键回退路径、Bus ID 探测与默认回退已覆盖；并已补真实 stdin/stdout 样例覆盖“自动检测失败后切手工模式”和“深层菜单返回”
  剩余风险：如果后续还要继续收，只剩非法 Bus ID 输入之类的更细粒度终端样例
  下一步：把真实终端样例转到别的交互页

### 2.6 Server Override

- 场景：开启 / 关闭 override、逐项 bool 选择、返回 summary
  当前状态：回退逻辑已覆盖；并已补真实 stdin/stdout 样例覆盖“启用 override 后逐项布尔问答”和“沿用主机现有配置不会进入逐项问答”
  剩余风险：如果后续还要继续收，只剩跨步骤回退后的更长链路终端样例
  下一步：把真实终端样例转到别的交互页

### 2.7 Summary / Execute

- 场景：确认同步、确认重建、退出
  当前状态：流程级测试已覆盖主干，`continue / back / quit` 输入映射已覆盖；并已补一条真实 stdin/stdout 样例覆盖“两个确认提示 + cleanup 失败后的最终错误文本”
  剩余风险：如果后续还要继续收，只剩更长链路的真实终端样例
  下一步：把真实终端样例转到尚未覆盖的其他交互页

## 3. 剩余命令与探测语义清单

下面这张清单只记录仍值得继续收的点，不重复已经收好的主链。

### 3.1 强依赖命令

- `[release.rs](../mcbctl/src/bin/control/mcb-deploy/release.rs)`
  命令：`gh auth status`、`git status --porcelain`、`git rev-parse <tag>`、`git push`、`gh release create`、`gh workflow run`
  当前语义：失败直接中止
  剩余工作：`dirty worktree / allow_dirty / probe failure / git push / gh release create / gh workflow run` 失败传播测试都已补；后续如果还要继续收，只剩更接近真实 CLI 的端到端样例

- `[orchestrate/env.rs](../mcbctl/src/bin/control/mcb-deploy/orchestrate/env.rs)`
  命令：`cargo check`
  当前语义：失败直接中止；缺 `cargo` 只告警
  剩余工作：这条边界和最小测试都已补；后续如果还要继续收，只剩更接近真实终端输出的样例

### 3.2 增强型探测命令

- `[orchestrate/dns.rs](../mcbctl/src/bin/control/mcb-deploy/orchestrate/dns.rs)`
  命令：`ip route show default`
  当前语义：`ip` 缺失时静默回退；命令异常或输出无效时显式告警后回退
  剩余工作：如果后续需要，再补更接近终端输出的集成样例

- `[execute.rs](../mcbctl/src/bin/control/mcb-deploy/execute.rs)`
  命令：`date +%Y%m%d-%H%M%S`
  当前语义：命令失败或输出为空会显式告警，并回退到 `unknown`
  剩余工作：如果后续需要，再补更接近真实终端输出的样例

- `[source/local.rs](../mcbctl/src/bin/control/mcb-deploy/source/local.rs)`
  命令：`git rev-parse HEAD`
  当前语义：命令失败或输出为空会显式告警，继续复制本地源，并清空旧的 `source_commit`
  剩余工作：这条语义已经收口；后续只需在需要时补更接近真实终端输出的样例

- `[source/remote.rs](../mcbctl/src/bin/control/mcb-deploy/source/remote.rs)`
  命令：`git rev-parse HEAD`
  当前语义：命令失败或输出为空会显式告警，继续保留已成功拉取的远端源，并清空旧的 `source_commit`
  剩余工作：如果后续需要，再补更接近真实终端输出的 clone + probe 样例

### 3.3 权限与环境探测

- `[orchestrate/env.rs](../mcbctl/src/bin/control/mcb-deploy/orchestrate/env.rs)`
  命令：`id -u`
  当前语义：`id` 缺失或探测异常会显式告警，并按非 root 环境继续
  剩余工作：如果后续还要继续收，重点应转向 backup 时间戳和本地源 commit 探测，不再是 `id -u`

### 3.4 清理与辅助路径

- `[utils.rs](../mcbctl/src/bin/control/mcb-deploy/utils.rs)`
  路径：`can_write_dir()`
  当前语义：保留 best-effort；探针文件删除失败不会升级成业务错误
  剩余工作：这条语义和最小测试都已补；后续如果还要继续收，只剩更接近真实 rootless 终端环境的样例

- `[scaffold/users.rs](../mcbctl/src/bin/control/mcb-deploy/scaffold/users.rs)`
  路径：`MCBCTL_COPY_USER_TEMPLATE`
  当前语义：只有字面值 `true` 开启；缺失、无效值或读取失败都等价于关闭
  剩余工作：这条语义和最小测试都已补；后续如果还要继续收，只剩更接近真实脚手架样例的端到端验证

## 4. 最小测试夹具策略

接下来不应该先大重构，而应该只做最小夹具收敛。

原则：

- 不引入重量级 pseudo-terminal 框架，除非现有 seam 明确不够
- 先复用现有 `WizardFlowRunner` / `DeployFlowRunner`
- 只有当 prompt 输入输出本身成了盲区，才加一层最小 `PromptIo` seam
- `test_app(...)` 的收敛应只覆盖重复字段初始化，不碰业务逻辑

建议顺序：

1. 先补 prompt 级输入映射测试
2. 再补 1-2 条真实 stdin/stdout 级样例
3. 如果样例价值明显高，再考虑扩大交互测试覆盖

## 5. 分批实施顺序

下一阶段按下面顺序推进：

1. `P0`：TTY 输入矩阵
2. `P1`：剩余增强型探测命令语义
3. `P1`：权限/环境探测的一致化
4. `P2`：测试夹具收敛
5. 文档同步收尾

## 6. 验收标准

每一批继续推进时，都至少要满足：

- `cargo fmt --check --manifest-path mcbctl/Cargo.toml`
- `cargo clippy --manifest-path mcbctl/Cargo.toml --all-targets --all-features -- -D warnings`
- `cargo test --manifest-path mcbctl/Cargo.toml`
- `NIX_CONFIG='experimental-features = nix-command flakes' nix flake check --option eval-cache false`
- `NIX_CONFIG='experimental-features = nix-command flakes' nix run .#mcbctl -- repo-integrity`

针对交互批次，还要额外满足：

- 至少新增一组 prompt 级输入映射测试
- 至少新增一组跨步骤返回/重试测试
- 不新增静默降级路径
