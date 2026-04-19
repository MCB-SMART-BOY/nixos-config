# `mcbctl` 主线 UX 收敛方案

这份文档描述的是“把当前 `mcbctl` 收敛到接近 `lazygit` 的日用体验”应该怎么做。

它不是实现记录，而是产品级方案。

如果需要去二义性的实现规格，请直接看 [UX_SPEC_CN.md](./UX_SPEC_CN.md)。

## 1. 目标

目标不是把 `mcbctl` 做成 Git 工具，而是把它做成“这个仓库的日用控制台”。

要达到的体验是：

1. 用户一进入就知道当前状态
2. 用户能用最少决策完成最常见任务
3. 高级能力仍然完整，但不会压在主路径上

## 2. 非目标

这些不是当前阶段要追求的：

- 不追求和 `lazygit` 的界面外观相似
- 不追求一次性重写现有 TUI 全部页面
- 不追求删除现有完整 deploy wizard
- 不追求把所有高级配置隐藏到不可达

## 3. 核心判断

当前 `mcbctl` 的问题不是能力不足，而是默认暴露的信息密度过高。

所以主线改造的方向应该是：

- 从“功能页中心”改成“任务中心”
- 从“专家参数优先”改成“日用主路径优先”
- 从“先选细节，再执行”改成“先看状态，再进入默认动作”

## 4. 目标用户与高频任务

### 4.1 主要用户

- 仓库维护者本人
- 偶尔需要修改 host/user 配置的日用使用者
- 偶尔需要走高级部署、release、模板脚手架的专家用户

### 4.2 最高频任务

这 5 个任务应当决定首页和主路径：

1. 看当前 host / repo 是否健康
2. 看哪里有未保存修改
3. 预览“应用到当前主机”会做什么
4. 执行“应用到当前主机”
5. 进入高级编辑或高级部署

只要这 5 个任务还不顺，`mcbctl` 就还不能算接近日用控制台。

## 5. 新信息架构

建议把顶层结构收成 5 个区域：

- `Overview`
- `Edit`
- `Apply`
- `Advanced`
- `Inspect`

### 5.1 Overview

职责：

- 显示当前 host
- 显示 repo 健康和环境健康
- 显示 dirty / unsaved 状态
- 提供最常用主动作入口

### 5.2 Edit

职责：

- 承接现有 `Packages / Home / Users / Hosts`
- 继续保留受管写回和当前页校验
- 强化当前页摘要、dirty、validate/save 状态
- 在壳层直接显示 dirty 子页，不要求用户先切进去才知道哪一页未保存
- 保持四页边界，但把页标题和当前目标对象绑定，减少跨页切换时的上下文重建

### 5.3 Apply

职责：

- 专门承接“应用到当前主机”
- 显示统一预览
- 执行当前 host 的默认安全路径

### 5.4 Advanced

职责：

- 承接完整 deploy wizard
- 承接新 host / 新用户 / source strategy / release
- 承接 runtime override 细项

### 5.5 Inspect

职责：

- repo-integrity / doctor / lint
- 预览命令
- 最近一次执行结果
- 错误与阻塞项说明

## 6. 首页布局

首页不该再是“功能菜单”，而应该是“状态总览 + 主动作”。

建议布局：

```text
+---------------------------------------------------------------+
| mcbctl                                                        |
| Host: nixos   Profile: desktop   Mode: switch   Privilege: sudo |
+---------------------------------------------------------------+
| Repo Health            | Unsaved Changes                      |
| repo-integrity: ok     | Packages: clean                      |
| doctor: warning(1)     | Home: dirty                          |
| flake-check: clean     | Users: clean                         |
| branch: rust脚本分支   | Hosts: clean                         |
+---------------------------------------------------------------+
| Current Source         | Last Action                          |
| local repo             | rebuild switch succeeded             |
| commit: abcdef0        | 2026-04-14 13:40                     |
+---------------------------------------------------------------+
| Overview Summary                                              |
| status: can preview apply   next: open Apply preview          |
+---------------------------------------------------------------+
| Secondary Actions                                             |
| [Open Edit] [Advanced] [Inspect] [Help]                      |
+---------------------------------------------------------------+
```

### 6.1 首页必须回答的问题

用户进入后 5 秒内必须知道：

- 当前操作的 host 是谁
- 当前仓库是否健康
- 有没有未保存内容
- 现在能不能直接应用
- 如果不能，卡在哪

## 7. 默认主路径

默认主路径应该固定成：

```text
Overview
  -> Preview Apply
  -> 检查 dirty / 校验 / source / target / privilege
  -> 统一预览
  -> Apply Current Host
  -> 成功 / 失败摘要
```

### 7.1 用户默认不该先选这些东西

这些都不该出现在日用第一层：

- source strategy 细节
- remote pin / HEAD
- 新 host / 新用户脚手架
- GPU Bus ID
- per-user TUN 细项
- server override
- release 参数

它们应该全部后移到 `Advanced`。

## 8. Apply 页设计

`Apply` 不是现有 wizard 的别名，而是新的日用入口。

### 8.1 Apply 页展示内容

- target host
- source type
- rebuild mode
- root / sudo / rootless 状态
- 将写入哪些 managed 文件
- 是否会同步 `/etc/nixos`
- 是否要求真实 hardware config
- 关键命令预览
- 当前阻塞项

### 8.2 Apply 页动作

- `Apply Current Host`
- `Enter 进入 Advanced`
- 顶层 shell 切页

### 8.3 Apply 页行为

- 如果存在 dirty 页面，优先引导去保存或统一保存
- 如果存在阻塞项，`Apply` 不可执行，只给出下一步建议
- 如果只是 warning，允许继续，但要明确显示

## 9. Advanced 区设计

现有完整 wizard 不删，但角色已经变化：

- 不再是默认部署入口
- 作为 `Advanced` 下的专家路径保留

建议包含：

- `Advanced`
- `Create Host`
- `Create User`
- `Pinned Remote Source`
- `Runtime Overrides`
- `Release`

也就是“保留能力，但默认后置”。

## 10. Edit 区映射

现有页面不必立刻拆掉，但应改成任务化包装。

### 10.1 页面映射表

| 现有页面 | 新归属 | 默认可见性 | 说明 |
|---|---|---:|---|
| `Packages` | `Edit` | 高 | 保留 |
| `Home` | `Edit` | 高 | 保留 |
| `Users` | `Edit` | 中 | 保留 |
| `Hosts` | `Edit` | 中 | 保留 |
| `Deploy` | `Apply` | 高 | 日用路径收口到这里 |
| `Actions` | 历史过渡页 | 低 | 已拆除；动作已回到 `Inspect / Advanced`，`Apply` 只保留内部执行链 |

### 10.2 `Actions` 拆分建议

历史 `Actions` 已经拆散回归宿页；长期职责拆分保持为：

- `Inspect`
  - repo-integrity
  - lint-repo
  - doctor
  - flake check
- `Apply`
  - `Apply Preview`
  - `Apply Current Host`
- `Advanced`
  - flake update
  - update upstream pins
  - full deploy wizard

## 11. 交互规范

要接近 `lazygit`，不是靠颜色，而是靠一致性。

建议统一：

- `Enter`
  默认主动作
- `Tab`
  顶层区域切换
- `e`
  编辑
- `p`
  预览
- `a`
  应用
- `v`
  校验
- `s`
  保存
- `b`
  返回
- `q`
  退出
- `?`
  帮助

重点不是具体键，而是同一语义在各页保持一致。

帮助系统的默认形态也应统一：

- 页脚只保留当前页最常用的短提示
- 详细键位统一放进 `?` 帮助面板
- `Esc` 在非输入模式下优先关闭帮助面板，在输入模式下继续表示取消当前输入

## 12. 文案原则

外部文案应当优先任务语义，而不是内部实现语义。

建议：

- `source_ref`
  对外显示为 `Pinned Source`
- `allow_remote_head`
  对外显示为 `Track Latest`
- `per-user TUN`
  首页只显示 `User Tunnel: On/Off`
- `specialisations`
  首页只显示 `GPU Profiles`
- `override`
  首页只显示 `Advanced Overrides`

内部代码继续保持精确字段名，不必强行改模型。

## 13. 分期实施

### 阶段 1：信息架构收口

只做这些：

- 增加 `Overview / Apply / Advanced / Inspect` 概念
- 不删除现有页面
- 不大改业务逻辑

验收：

- 用户一进来能知道当前 host、dirty、health、主动作

### 阶段 2：默认主路径落地

只做这些：

- `Apply Current Host`
- `Preview Apply`
- 统一阻塞项显示

验收：

- 当前 host 日用部署从进入到确认不超过 3 个关键决策点

### 阶段 3：高级路径后移

只做这些：

- 完整 wizard 移入 `Advanced`
- `Actions` 拆到 `Apply / Inspect / Advanced`

验收：

- 普通用户不需要先进入高级向导才能完成常规更新

### 阶段 4：编辑区任务化

只做这些：

- 每个编辑页补状态摘要、dirty、validate/save
- 强化和 `Overview` 的联动

验收：

- 用户能清楚知道“改了什么、能不能保存、会不会阻止 apply”

## 14. 量化验收标准

如果要判断是否“足够接近 lazygit 风格主路径”，建议用这些标准：

1. 当前 host 常规 apply 不超过 3 个关键决策
2. 首页 5 秒内可回答“能不能部署”
3. 高级选项默认不出现
4. 所有 apply 前都有统一预览
5. 失败后都有下一步建议
6. 任一时刻都能看见：
   - 当前 host
   - 当前 dirty 状态
   - 当前可执行主动作

## 15. 当前建议的下一步

如果下一轮要开始真正落地，不建议直接改一堆 UI。

最稳的顺序是：

1. 先做 `Overview` 的信息模型
2. 再做 `Apply` 页的统一预览模型
3. 再把当前 `Deploy` 页折叠进 `Apply`
4. 最后再处理 `Actions` 和顶层导航收口
