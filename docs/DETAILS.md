# 项目细节与联动关系

`docs/STRUCTURE.md` 解决的是“去哪里改”，这页解决的是“为什么这样拆、改了以后会牵到哪里”。

如果你准备碰这些主题，这页最有价值：

- 多用户
- 用户包与系统包的边界
- GPU 配置与 specialisation
- Noctalia 和用户命令
- Rust 脚本如何接进整个仓库

## 快速定位

- 改默认用户、多用户、管理员：`hosts/<hostname>/default.nix` 与 `modules/users.nix`
- 改系统共享包组：`hosts/profiles/*.nix` 与 `modules/packages.nix`
- 改某个用户的软件：`home/users/<user>/packages.nix`
- 改用户界面配置：`home/users/<user>/config/`
- 改用户命令接线：`pkgs/mcbctl/default.nix` 与 `home/modules/desktop.nix`
- 改 TUI 机器管理区：`home/users/<user>/managed/` 与 `hosts/<host>/managed/`
- 改新用户模板：`home/templates/users/`
- 改主机模板：`hosts/templates/`
- 改控制台入口：`mcbctl/src/bin/mcbctl.rs`
- 改部署流程：`mcbctl/src/bin/control/mcb-deploy.rs`
- 改 Rust 公共逻辑：`mcbctl/src/lib.rs`

## 1. Home Manager 这一层，负责“人”

`home/` 这一层回答的是：

“某个用户登录之后，他看到的终端、编辑器、桌面栏、应用清单，应该是什么样？”

关键入口：

- `home/users/<user>/default.nix`
- `home/profiles/full.nix`
- `home/profiles/minimal.nix`
- `home/modules/*.nix`

如果某个用户的软件已经多到单个 `packages.nix` 很难维护，推荐继续在该用户目录里拆成：

- `home/users/<user>/packages.nix`
- `home/users/<user>/packages/*.nix`

也就是说，拆分发生在“这个用户自己的目录内部”，而不是再回到全局共享包模块。对于像 `mcbnixos` 这样的大用户目录，推荐直接做到“一个软件组一个文件”。

它最关键的设计点是：

- 每个用户都有自己的入口目录
- 每个用户都可以独立声明 `home.packages`
- 用户之间不再共享一份大而含混的软件清单

这就是为什么现在新增管理员用户、服务器用户、笔记本用户时，都应该给他们自己的 `home/users/<user>/packages.nix`。

## 2. 用户软件为什么要放 `home/users/<user>/packages.nix`

这不是风格问题，而是边界问题。

如果把 GUI 应用、聊天软件、办公软件、个人开发工具都继续扔进系统层，后面一定会出现这些问题：

- 多用户主机上，所有人都被同一份桌面软件清单绑住
- 很难看出哪些软件是机器共享的，哪些只是某个用户自己要的
- 删软件时不敢动，因为不知道会不会影响别人

现在这套仓库推荐的分工是：

- 系统层保留共享能力
- 用户层负责个人软件清单

Nix store 仍然会共享构建产物，区别只在于：

- 这个包是否出现在某个用户的 profile 里

## 3. 系统层，负责“机器”

系统层最核心的入口有三个：

- `hosts/<hostname>/default.nix`
- `hosts/profiles/*.nix`
- `modules/*.nix`

如果你改的是 `mcb.*` 选项本身，而不是消费这些选项的模块，
现在优先看 `modules/options/`；[options.nix](/home/mcbgaruda/projects/nixos-config/modules/options.nix) 已经降成聚合入口。

职责大致是：

- `hosts/<hostname>/default.nix`
  这台机器自己的决定，例如主机名、用户列表、管理员用户、主机私有覆盖
- `hosts/profiles/*.nix`
  某类机器默认该长什么样，例如桌面主机和服务器主机
- `modules/*.nix`
  跨主机复用的能力模块，例如用户、网络、GPU、服务、包组

## 4. 多用户模型真正靠哪几处起作用

关键字段主要在主机配置里：

- `mcb.user`
- `mcb.users`
- `mcb.adminUsers`

大体规则可以这样记：

- `mcb.user`
  主用户，很多默认路径和兼容行为会以它为中心
- `mcb.users`
  参与 Home Manager 管理的用户列表
- `mcb.adminUsers`
  具有管理员权限的用户列表

真正负责把这些用户落成系统用户、用户组和授权的，是：

- `modules/users.nix`

所以“加用户”不能只在 `home/users/` 里新建目录。
用户目录决定的是这个用户怎么用系统，不决定系统里有没有这个用户。

## 5. GPU 这块，仓库的假设是什么

GPU 配置集中在：

- `modules/hardware/gpu.nix`

关键字段包括：

- `mcb.hardware.gpu.mode`
- `mcb.hardware.gpu.igpuVendor`
- `mcb.hardware.gpu.prime.mode`
- `mcb.hardware.gpu.prime.intelBusId`
- `mcb.hardware.gpu.prime.amdgpuBusId`
- `mcb.hardware.gpu.prime.nvidiaBusId`
- `mcb.hardware.gpu.nvidia.open`
- `mcb.hardware.gpu.specialisations.*`

仓库当前不是单纯切一个布尔开关，而是明确支持：

- `igpu`
- `hybrid`
- `dgpu`

现在这块还有一个明确的部署期约束：

- `mcb-deploy` 会先识别当前主机属于单集显、多显卡还是独显主机
- 新建桌面主机时，不再依赖模板里硬编码的 GPU busId
- 真正机器相关的 GPU 参数会写进 `hosts/<hostname>/local.nix`

其中最容易踩坑的是 `hybrid`：

- 需要正确的 busId
- 需要 BIOS / MUX 支持
- 需要理解当前机器是不是已经被锁成 `dGPU-only`

`mcb-deploy` 在向导里会优先尝试自动探测 busId，探测不到才回退到主机现有配置。

## 6. Noctalia 与用户命令，现在是怎么接起来的

这一块以前最容易被误解成“几个零碎脚本”，现在其实已经很清楚了：

- Rust 命令实现放在 `mcbctl/src/bin/*.rs`
- Nix 打包入口在 `pkgs/mcbctl/default.nix`
- 桌面用户通过 `home/modules/desktop.nix` 把 `mcbctl` 包放进自己的环境

同时，仓库现在新增了机器管理区：

- `home/users/<user>/managed/`
- `hosts/<host>/managed/`

原则是：

- 手写配置继续留在你自己的文件里
- TUI / 自动化工具只写 `managed/`

当前已经接入的第一块就是：

- `mcbctl` 的 `Packages` 页面现在以 `nix search` 结果为主，`catalog/packages/*.toml` 只保留本地覆盖层和仓库内自维护包元数据
- 然后把勾选结果按组写入 `home/users/<user>/managed/packages/*.nix`
- 文件里会带 `managed-id` 标记，方便 TUI 下次直接读回

也就是说，当前状态已经不是：

- 某个用户目录里堆一堆原始 Shell 脚本

而是：

- 仓库统一维护一套 Rust 二进制
- 用户侧决定要不要把哪些命令链接进自己的环境

这也是为什么你现在去找 `home/users/<user>/scripts/`，已经不应该再找到真实实现了。

## 7. 部署流程现在怎么理解

部署流程的主入口现在是：

- `mcbctl/src/bin/control/mcb-deploy.rs`

它负责的事情包括：

- 环境检查
- 源码准备
- 仓库自检
- 目标主机与用户收集
- `/etc/nixos` 同步
- `nixos-rebuild` 重建
- release 发布

它内部现在也开始按职责继续拆：

- `mcbctl/src/bin/control/mcb-deploy/plan.rs`
  部署摘要、计划对象生成和同步/重建命令预览
- `mcbctl/src/bin/control/mcb-deploy/wizard.rs`
  交互式部署向导的步骤流转与回退
- `mcbctl/src/bin/control/mcb-deploy/execute.rs`
  `/etc/nixos` 备份、同步与 `nixos-rebuild` 执行
- `mcbctl/src/bin/control/mcb-deploy/selection.rs`
  主机/用户/管理员选择、模板解析和基础校验
- `mcbctl/src/bin/control/mcb-deploy/runtime.rs`
  运行时配置聚合入口
- `mcbctl/src/bin/control/mcb-deploy/runtime/`
  per-user TUN、GPU、服务器运行时能力配置分拆层
- `mcbctl/src/store/hosts/`
  主机 managed 存储分拆层；`eval.rs`、`layout.rs`、`render.rs` 分别负责评估读取、目录布局和 Nix 分片渲染写入
- `mcbctl/src/tui/state/model.rs`
  `AppContext` / `AppState` 模型定义与基础状态构造
- `mcbctl/src/bin/control/mcb-deploy/ui.rs`
  基础交互输出、菜单和确认提示
- `mcbctl/src/bin/control/mcb-deploy/orchestrate.rs`
  部署编排聚合入口
- `mcbctl/src/bin/control/mcb-deploy/orchestrate/`
  环境检查、仓库自检、临时 DNS、部署编排与总执行入口分拆层
- `mcbctl/src/bin/control/mcb-deploy/utils.rs`
  仓库探测、临时路径、复制与校验等通用工具函数
- `mcbctl/src/bin/control/mcb-deploy/scaffold.rs`
  新 host / 新用户目录脚手架与 `local.nix` 生成
- `mcbctl/src/bin/control/mcb-deploy/source.rs`
  配置来源、本地仓库探测、远端仓库拉取与镜像重试
- `mcbctl/src/bin/control/mcb-deploy/release.rs`
  release 版本解析、说明生成与发布

这里有一个很重要的变化：

- 仓库不再依赖 `run.sh` 和那套分层 Shell 脚本
- 部署与 release 逻辑现在都统一在 Rust 里维护

## 8. Zed 和 YesPlayMusic 为什么仍然单独打包

它们现在仍然走自定义包目录：

- `pkgs/zed/`
- `pkgs/yesplaymusic/`

目的不是为了复杂化，而是为了控制两个东西：

- 上游版本
- 固定 hash

相关更新入口现在也已经统一成 Rust 工具：

- `update-zed-source`
- `update-yesplaymusic-source`
- `update-upstream-apps`

在仓库根目录里，推荐直接这样跑：

```bash
nix run .#update-upstream-apps
```

## 9. 图形运行时补丁为什么存在

桌面环境里总会碰到一类程序：

- 不是从 nixpkgs 来的二进制
- 或者是上游自己打包的应用
- 或者是 AppImage / 官网 tarball / 语言工具链自行下载的程序

它们常见的问题是：

- Vulkan ICD 找不到
- 图形栈依赖继承不稳定
- `LD_LIBRARY_PATH` 或运行时环境不完整

这套仓库把相关补丁集中放在桌面模块里收口，而不是让每个上游程序都各自打一套补丁。

## 10. 维护时最值得坚持的习惯

如果你想让这套仓库长期还能舒服维护，最值钱的不是多会写 Nix，而是下面这几个习惯：

- 改系统层之前，先确认这是不是用户层问题
- 改用户软件之前，先确认这是不是系统共享能力
- 改脚本逻辑时，先看 `mcbctl/src/lib.rs` 里有没有现成复用函数
- 改 Noctalia / 桌面按钮时，不要只看配置文件，也要看命令入口是不是已经接好

这几个习惯会直接决定仓库是越来越清楚，还是越改越粘。
