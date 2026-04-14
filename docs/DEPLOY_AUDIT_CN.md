# 部署主线审计基线

这份文档不是历史说明，而是接下来继续收 `mcb-deploy` / `mcbctl deploy` 主线时的工作基线。

如果要从“工程主线”继续往“日用体验主线”推进，产品级方案见 [UX_MAINLINE_CN.md](/home/mcbgaruda/projects/nixos-config/docs/UX_MAINLINE_CN.md)，去二义性的实现规格见 [UX_SPEC_CN.md](/home/mcbgaruda/projects/nixos-config/docs/UX_SPEC_CN.md)。

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
  当前状态：非 TTY 默认、prepare failure 后 `retry / reselect / exit`、非法输入重试、固定 ref 留空重试 已有测试
  剩余风险：真实 stdin/stdout 级样例仍未补
  下一步：视收益决定是否补 1 条真实终端样例

- 场景：来源准备失败后重新选择策略
  当前状态：流程级测试已覆盖
  剩余风险：交互提示文本和内部动作映射是否完全一致
  下一步：补“输入 1/2/3/q”与动作映射测试

### 2.2 Host 选择

- 场景：已有 host 选择、新建 host、`UpdateExisting` 禁止新建
  当前状态：默认 host、保留目录过滤、TTY 非法输入重试、新建 host 名称重试 已覆盖
  剩余风险：TTY 下重选 host 后，旧 host 的 profile / GPU / TUN / server override 状态是否在真实交互里完全清掉
  下一步：补“host 重选 -> users/admin/runtime 再进入”的交互回归

### 2.3 Users / Admin

- 场景：选择已有用户、新增用户、清空用户、返回上一页
  当前状态：非 TTY 默认、非法用户名重试、清空后禁止直接完成 已覆盖
  剩余风险：从 admin 返回 users 再修改时默认管理员是否同步收缩；“空输入取消”仍未直接覆盖
  下一步：补 users/admin 连续往返测试

- 场景：管理员选择
  当前状态：默认管理员、清空后禁止直接完成 已覆盖
  剩余风险：用户列表变化后旧管理员残留在真实交互里是否被完全清理
  下一步：补 users/admin 连续往返测试

### 2.4 per-user TUN

- 场景：开启 / 关闭、为每个用户填写接口和 DNS 端口
  当前状态：状态机回退已覆盖
  剩余风险：非法端口、空输入默认、用户数变化后旧映射残留、返回时字段清理是否和 summary 一致
  下一步：补逐用户输入恢复测试

### 2.5 GPU

- 场景：自动检测、手工模式、Bus ID 选择 / 手填、specialisation 选择
  当前状态：非 TTY 默认、关键回退路径、Bus ID 探测与默认回退已覆盖
  剩余风险：多层菜单下 `Back` 连续返回、手填 Bus ID 非法、自动检测失败后切手工模式的真实输入流
  下一步：补 GPU 多层交互的返回矩阵

### 2.6 Server Override

- 场景：开启 / 关闭 override、逐项 bool 选择、返回 summary
  当前状态：回退逻辑已覆盖
  剩余风险：多次开关后旧字段残留、从 summary 返回 override 后再关闭时的状态清理
  下一步：补 “summary -> server override -> summary” 循环测试

### 2.7 Summary / Execute

- 场景：确认同步、确认重建、退出
  当前状态：流程级测试已覆盖主干，`continue / back / quit` 输入映射已覆盖
  剩余风险：确认后 cleanup 失败时的终端表现
  下一步：视收益决定是否补 1 条真实终端样例

## 3. 剩余命令与探测语义清单

下面这张清单只记录仍值得继续收的点，不重复已经收好的主链。

### 3.1 强依赖命令

- `[release.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/release.rs)`
  命令：`gh auth status`、`git rev-parse <tag>`、`git push`、`gh release create`、`gh workflow run`
  当前语义：失败直接中止
  剩余工作：补更细的失败传播测试，而不是改语义

- `[orchestrate/env.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/orchestrate/env.rs)`
  命令：`cargo check`
  当前语义：失败直接中止；缺 `cargo` 只告警
  剩余工作：确认这是否继续保持“增强型检查”而非硬依赖

### 3.2 增强型探测命令

- `[orchestrate/dns.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/orchestrate/dns.rs)`
  命令：`ip route show default`
  当前语义：`ip` 缺失时静默回退；命令异常或输出无效时显式告警后回退
  剩余工作：如果后续需要，再补更接近终端输出的集成样例

- `[execute.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/execute.rs)`
  命令：`date +%Y%m%d-%H%M%S`
  当前语义：时间戳失败会回退到 `backup`
  剩余工作：决定是否需要像 release 一样显式告警

- `[source/local.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/source/local.rs)`
  命令：`git rev-parse HEAD`
  当前语义：失败只是不显示 `source_commit`
  剩余工作：确认这是允许的静默增强，还是应该告警后继续

### 3.3 权限与环境探测

- `[orchestrate/env.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/orchestrate/env.rs)`
  命令：`id -u`
  当前语义：`id` 缺失或探测异常会显式告警，并按非 root 环境继续
  剩余工作：如果后续还要继续收，重点应转向 backup 时间戳和本地源 commit 探测，不再是 `id -u`

### 3.4 清理与辅助路径

- `[utils.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/utils.rs)`
  路径：`can_write_dir()`
  当前语义：探针文件删除失败会静默吞掉
  剩余工作：判断是否保留 best-effort；如果保留，要在文档里明确这是探针清理而非业务清理

- `[scaffold/users.rs](/home/mcbgaruda/projects/nixos-config/mcbctl/src/bin/control/mcb-deploy/scaffold/users.rs)`
  路径：`MCBCTL_COPY_USER_TEMPLATE`
  当前语义：环境变量读取失败等价于关闭
  剩余工作：低优先级；这里更像配置开关，不建议过度收紧

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
