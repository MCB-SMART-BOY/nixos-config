# 项目结构说明

如果你打开仓库后的第一反应是“我到底该改哪一层”，这页就是给这个时刻准备的。

这套仓库现在的组织原则很简单：

- `hosts/` 和 `modules/` 管机器
- `home/users/` 管人
- `mcbctl/` 管脚本逻辑
- `pkgs/` 管仓库自己维护的包

## 一眼先记住这些入口

- 主机入口：`hosts/<hostname>/default.nix`
- 主机模板：`hosts/templates/`
- 系统模块：`modules/`
- 用户入口：`home/users/<user>/default.nix`
- 用户软件：`home/users/<user>/packages.nix`
- 用户模板：`home/templates/users/`
- 用户命令打包：`pkgs/mcbctl/default.nix`
- 用户机器管理区：`home/users/<user>/managed/`
- Rust 脚本集合：`mcbctl/`
- Rust 脚本打包：`pkgs/mcbctl/`
- 软件目录本地覆盖层：`catalog/packages/*.toml`
- 软件组元数据：`catalog/groups.toml`
- Home 结构化选项元数据：`catalog/home-options.toml`

如果你只记住这些位置，已经足够处理大部分维护工作。

## 顶层目录怎么理解

```text
.
├── flake.nix
├── flake.lock
├── configuration.nix
├── mcbctl/
├── hosts/
├── modules/
├── home/
├── catalog/
├── pkgs/
├── docs/
└── README.md
```

### `hosts/`

这里回答的是：“这台机器是谁？”

常见内容：

- `hosts/<hostname>/default.nix`
  主机入口，决定这台机器导入哪个 profile、默认用户是谁、有哪些主机级覆盖
- `hosts/<hostname>/managed/`
  给 `mcbctl` / 自动化工具写入的主机管理区；现在按 `users.nix`、`network.nix`、`gpu.nix`、`virtualization.nix` 分片
- `hosts/<hostname>/system.nix`
  机器架构，例如 `"x86_64-linux"`
- `hosts/<hostname>/local.nix`
  主机私有覆盖

硬件配置文件不再放在 `hosts/<hostname>/` 里，而是统一使用仓库根目录的 `hardware-configuration.nix`。
实际部署到机器上时，它对应 `/etc/nixos/hardware-configuration.nix`，和 `/etc/nixos/configuration.nix` 同级。

### `hosts/profiles/`

这里回答的是：“这台机器大体属于哪类角色？”

例如：

- `hosts/profiles/desktop.nix`
- `hosts/profiles/server.nix`

它们是组合好的主机模板，不是某一台具体机器。

### `hosts/templates/`

这里放“拿来复制”的主机模板。

这些目录不会被 flake 当成真实主机扫描，也不应该直接拿来部署。
它们的作用是：

- 作为新主机目录的起点
- 保存桌面 / 服务器这类较完整的主机样板
- 避免示例主机继续污染真实 `nixosConfigurations`

### `modules/`

这里放系统层公共能力。

例如：

- 用户与权限
- 网络与代理
- GPU
- 服务
- 系统共享包组

其中 `mcb.*` 选项定义现在已经进一步拆到 `modules/options/`，
而 [options.nix](/home/mcbgaruda/projects/nixos-config/modules/options.nix) 只保留聚合入口。

如果一个改动应该影响一整类主机，而不是某个具体用户，通常就在这里。

### `home/`

这里放 Home Manager，也就是“某个用户登录后会看到什么”。

常见子目录：

- `home/profiles/`
  用户配置组合，例如完整桌面用户、最小服务器用户
- `home/modules/`
  用户层公共模块
- `home/users/<user>/`
  某个具体用户自己的入口目录
- `home/templates/users/`
  新用户模板，给 `mcb-deploy` 或手工复制使用

### `home/users/<user>/`

这是你最常改的地方之一。

常见文件：

- `default.nix`
  用户入口
- `packages.nix`
  这个用户的软件声明
- `packages/`
  当某个用户的软件清单过大时，可继续在用户目录内拆成“一个软件组一个文件”
- `managed/`
  机器管理区，保留给 TUI / 自动化工具写入
  现在用户设置会进一步拆到 `managed/settings/desktop.nix`、`session.nix`、`mime.nix`
- `local.nix`
  不想进仓库的私有覆盖
- `local.nix.example`
  起步示例
- `config/`
  会被链接到 `~/.config`
  其中 shell 通用模块继续拆在 `home/modules/shell/*.nix`，fish 交互层按 `config/fish/conf.d/*.fish` 拆分；函数层也继续按 `core` / `navigation` / `nixos` / `file` 分开，不再把所有逻辑塞进单个文件
- `assets/`
  这个用户自己的资源文件

这里最重要的变化是：

- 用户命令现在不再从 `home/users/<user>/scripts/` 读取原始 Shell 脚本
- 而是通过 `mcbctl` 编译出来的二进制、再由 `pkgs/mcbctl/` 暴露到环境里
- 用户软件现在也可以通过 `managed/packages/*.nix` 由 `mcbctl` 的 `Packages` 页面写入，`managed/packages.nix` 只做聚合导入

### `home/templates/users/`

这里放用户模板，而不是实际启用的用户。

目前的使用方式是：

- `mcb-deploy` 会按主机类型优先从这里挑模板
- 模板主要提供 `packages.nix` 这类默认内容
- 真正生效的用户入口仍然应该落在 `home/users/<user>/`

### `mcbctl/`

这里放 Rust 写的脚本实现。

常见内容：

- `mcbctl/src/bin/*.rs`
  现在按领域归类到子目录里：
  `control/`、`network/`、`desktop/`、`music/`、`noctalia/`、`update/`
- `mcbctl/src/lib.rs`
  公共函数和复用逻辑
- `mcbctl/src/domain/`
  TUI / 控制台共享的数据模型和枚举；`DeployPlan` 这类跨入口复用的部署计划对象也在这里
- `mcbctl/src/store/`
  读写 `catalog/`、`managed/`、主机探测这类存储与环境逻辑；`store/deploy.rs` 现在也承载共享的同步、硬件配置生成与 `nixos-rebuild` 执行
- `mcbctl/src/tui/views/`
  TUI 渲染层；已按页面拆开，不再继续把所有渲染堆在一个 `mod.rs` 里
- `mcbctl/src/tui/state.rs`
  当前保留的顶层状态机与跨页编排层
- `mcbctl/src/tui/state/`
  页面级状态逻辑分拆层；`deploy.rs`、`packages.rs`、`home.rs`、`actions.rs`、`hosts.rs` 已经从主 `state.rs` 中拆出
- `mcbctl/src/tui/state/hosts/`
  `Users` / `Hosts` 两页的进一步分拆层；用户结构编辑和运行时配置编辑分别落到 `users.rs`、`runtime.rs`
- `mcbctl/src/tui/state/packages/`
  `Packages` 页的进一步分拆层；浏览/汇总和交互修改分别落到 `browse.rs`、`mutate.rs`

部署入口本身也开始按职责拆分：

- `mcbctl/src/bin/control/mcb-deploy.rs`
  部署向导主入口与高层编排
- `mcbctl/src/bin/control/mcb-deploy/plan.rs`
  部署摘要、`DeployPlan` 拼装和 `nixos-rebuild` / repo sync 计划对象生成
- `mcbctl/src/bin/control/mcb-deploy/wizard.rs`
  交互式部署向导的步骤流转与回退逻辑
- `mcbctl/src/bin/control/mcb-deploy/execute.rs`
  `/etc/nixos` 备份、同步与重建执行
- `mcbctl/src/bin/control/mcb-deploy/selection.rs`
  主机/用户/管理员选择、模板解析与相关校验
- `mcbctl/src/bin/control/mcb-deploy/runtime.rs`
  运行时配置聚合入口
- `mcbctl/src/bin/control/mcb-deploy/runtime/`
  per-user TUN、GPU、服务器运行时能力配置分拆层；`gpu.rs`、`tun.rs`、`server.rs` 分别承载各自的交互与状态写入
- `mcbctl/src/bin/control/mcb-deploy/scaffold.rs`
  新 host / 新用户目录脚手架与 `local.nix` 生成
- `mcbctl/src/bin/control/mcb-deploy/source.rs`
  来源准备、本地仓库/远端仓库选择与拉取
- `mcbctl/src/bin/control/mcb-deploy/release.rs`
  release 版本解析、说明生成与发布流程
- `mcbctl/src/bin/noctalia/noctalia-gpu-mode.rs`
  Noctalia GPU 模式入口与参数分发
- `mcbctl/src/bin/noctalia/noctalia-gpu-mode/state.rs`
  当前模式探测、specialisation 列表与状态文件读写
- `mcbctl/src/bin/noctalia/noctalia-gpu-mode/menu.rs`
  GUI/CLI 菜单选择流程
- `mcbctl/src/bin/noctalia/noctalia-gpu-mode/apply.rs`
  切换命令拼装与终端执行

这里现在不只是“备用路线”，而是仓库的正式脚本实现。

### `catalog/`

这里放给 `mcbctl` 使用的本地覆盖层与目录元数据，而不是实际安装结果。

当前已经拆成三类：

- `catalog/packages/*.toml`
  仓库内自维护包与少量本地覆盖条目；`Packages` 页真正的大头来源已经转向 `nix search`
- `catalog/groups.toml`
  软件组标签、说明和排序，决定 `Packages` 页如何展示和排序组
- `catalog/home-options.toml`
  `Home` 页结构化选项的标签、说明和顺序

后面如果 `Packages` / `Home` 页继续长功能，优先往这里补元数据，而不是先把文案硬编码进 TUI。

### `pkgs/`

这里放仓库自己维护的包和包装逻辑。

现在比较关键的是：

- `pkgs/mcbctl/`
- `pkgs/zed/`
- `pkgs/yesplaymusic/`
- `catalog/packages/`

如果你想追官网稳定版，或者把仓库内部工具做成 Nix 包，这里就是核心位置。

## 按“我要改什么”来定位

### 我要改主机名、默认用户、管理员用户

看：

- `hosts/<hostname>/default.nix`

### 我要改系统级共享软件或服务

看：

- `hosts/profiles/*.nix`
- `modules/packages.nix`
  其中系统包模块已经继续拆到 `modules/packages/options.nix`、`system.nix`、`group-defs.nix`、`suites.nix`
- `modules/services/*.nix`
- `modules/networking.nix`

### 我要给某个用户加软件

看：

- `home/users/<user>/packages.nix`

### 我要改 Niri / Noctalia / 终端 / 编辑器配置

看：

- `home/users/<user>/config/`

### 我要改某个用户的命令入口

看：

- `pkgs/mcbctl/default.nix`
- `home/modules/desktop.nix`

### 我要改部署或追新工具

看：

- `mcbctl/src/bin/mcbctl.rs`
- `mcbctl/src/bin/control/mcb-deploy.rs`
- `mcbctl/src/bin/control/mcb-deploy/plan.rs`
- `mcbctl/src/bin/control/mcb-deploy/wizard.rs`
- `mcbctl/src/bin/control/mcb-deploy/execute.rs`
- `mcbctl/src/bin/control/mcb-deploy/selection.rs`
- `mcbctl/src/bin/control/mcb-deploy/runtime.rs`
- `mcbctl/src/bin/control/mcb-deploy/runtime/`
- `mcbctl/src/bin/control/mcb-deploy/scaffold.rs`
- `mcbctl/src/bin/control/mcb-deploy/source.rs`
- `mcbctl/src/bin/control/mcb-deploy/release.rs`
- `mcbctl/src/bin/update/`

## 最值得坚持的边界

一句话总结：

- 机器共享能力放系统层
- 用户个性化声明放用户层

只要这个边界不乱，仓库再大也还能读。
一旦这个边界开始混，后面每次改东西都会越来越累。
