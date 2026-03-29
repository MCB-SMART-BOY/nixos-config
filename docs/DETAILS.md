# 项目细节与联动关系

如果说 `docs/STRUCTURE.md` 解决的是“去哪里改”，那这页解决的就是“为什么这样拆、改了以后会连到哪里”。

它更像一张维护地图。
你不用一口气全部记住，但当你准备动下面这些地方时，这页会很有用：

- 多用户
- 用户包与系统包的边界
- GPU 配置与 specialisation
- Noctalia 按钮和用户脚本
- `run.sh` / `scripts-rs` 两套路线

---

## 快速定位

- 改默认用户、多用户、管理员：`hosts/<hostname>/default.nix` 与 `modules/users.nix`
- 改系统共享包组：`hosts/profiles/*.nix` 与 `modules/packages.nix`
- 改某个用户的软件：`home/users/<user>/packages.nix`
- 改用户界面配置：`home/users/<user>/config/`
- 改 Noctalia 行为：`home/users/<user>/noctalia.nix`、`home/users/<user>/scripts/`
- 改部署流程：`run.sh` 与 `scripts/run/`
- 改 Rust 脚本：`scripts-rs/`

---

## 1. Home Manager 这一层，负责“人”

你可以把 `home/` 理解成一个问题：

“某个用户登录之后，他看到的终端、编辑器、桌面栏、应用清单，应该是什么样？”

关键入口：

- `home/users/<user>/default.nix`
- `home/profiles/full.nix`
- `home/profiles/minimal.nix`
- `home/modules/*.nix`

比较常见的公共模块有：

- `home/modules/base.nix`
  - 环境变量、PATH、基础行为
- `home/modules/shell.nix`
  - zsh、direnv、zoxide、starship、tmux
- `home/modules/programs.nix`
  - 终端和编辑器等常见程序
- `home/modules/desktop.nix`
  - 桌面集成、Noctalia、桌面入口、图形运行时补丁
- `home/modules/git.nix`
  - git 的公共基础设置

这层最关键的设计点是：

- 每个用户都有自己的入口目录
- 每个用户都可以独立声明 `home.packages`
- 用户之间不通过“共享一个大文件”来分配软件

这意味着你以后新增管理员用户、服务器用户、笔记本用户时，都可以让他们拥有自己的软件声明空间，而不是一改全改。

---

## 2. 用户软件为什么要放 `home/users/<user>/packages.nix`

这不是风格问题，是边界问题。

以前最容易走歪的路线，是把 GUI 应用、聊天软件、办公软件、个人开发工具都塞进系统层。
短期看很方便，长期会出几个问题：

- 多用户主机上，所有人都被同一份桌面软件清单绑住
- 很难看出哪些软件是机器共享的，哪些只是某个用户自己要的
- 以后删软件时，不敢动，因为不知道会不会影响别人

现在这套仓库更推荐的分工是：

### 系统层保留共享能力

例如：

- shell 基础工具
- 网络工具
- Wayland 基础运行时
- 调试工具
- 代理服务
- 系统服务依赖

### 用户层负责个人软件清单

例如：

- Zed
- YesPlayMusic
- 浏览器
- 聊天软件
- 办公与科研软件
- 某个用户自己的开发工具

这样做时，Nix store 仍然会共享构建产物。
区别只在于“这个包是否出现在该用户的 profile 里”，而不是“机器上到底有没有重复装一份”。

---

## 3. 系统层，负责“机器”

系统层最核心的入口有三个：

- `hosts/<hostname>/default.nix`
- `hosts/profiles/*.nix`
- `modules/*.nix`

这三层各自的职责不一样：

### `hosts/<hostname>/default.nix`

这里放“这台机器自己的决定”。

例如：

- 主机名
- 主机角色
- 默认用户
- 用户列表
- 管理员用户
- 主机私有覆盖

### `hosts/profiles/*.nix`

这里放“某一类机器默认应该长什么样”。

例如：

- 桌面主机默认启用哪些包组
- 服务器默认关闭哪些图形能力
- 哪些系统模块默认导入

### `modules/*.nix`

这里放“跨主机复用的能力模块”。

例如：

- `modules/users.nix`
- `modules/packages.nix`
- `modules/networking.nix`
- `modules/nix.nix`
- `modules/services*.nix`
- `modules/hardware/gpu.nix`

---

## 4. 多用户模型，真正起作用的是哪几处

关键字段主要在主机配置里：

- `mcb.user`
- `mcb.users`
- `mcb.adminUsers`

大体规则可以这样理解：

- `mcb.user`
  - 主用户，很多兼容路径和默认行为会以它为中心
- `mcb.users`
  - 参与 Home Manager 管理的用户列表
- `mcb.adminUsers`
  - 具有管理员权限的用户列表

真正负责把这些用户落成系统用户、用户组和授权的，是：

- `modules/users.nix`

这也是为什么“加用户”不能只在 `home/users/` 里新建目录。
用户目录决定的是这个用户怎么用系统，不决定系统里有没有这个用户。

---

## 5. GPU 这块，仓库的假设是什么

GPU 配置集中在：

- `modules/hardware/gpu.nix`

关键字段是：

- `mcb.hardware.gpu.mode`
- `mcb.hardware.gpu.igpuVendor`
- `mcb.hardware.gpu.prime.mode`
- `mcb.hardware.gpu.prime.intelBusId`
- `mcb.hardware.gpu.prime.amdgpuBusId`
- `mcb.hardware.gpu.prime.nvidiaBusId`
- `mcb.hardware.gpu.nvidia.open`
- `mcb.hardware.gpu.specialisations.*`

仓库当前的设计不是单纯切一个布尔开关，而是明确支持：

- `igpu`
- `hybrid`
- `dgpu`

其中最容易踩坑的是 `hybrid`：

- 需要正确的 busId
- 需要 BIOS / MUX 支持
- 需要理解当前机器是不是本来就被锁成 dGPU-only

`run.sh` 的向导在配置 `hybrid` 时会做两件对实际使用很重要的事：

- 优先尝试从 `lspci` 自动探测 busId
- 探测不到时回退到主机现有配置

这意味着它不是瞎问一圈，而是在尽量复用你机器已经有的信息。

---

## 6. Noctalia 与用户脚本，是怎么接起来的

这一块容易误解成“只是几个桌面小脚本”，但实际上它有明确层次。

原始脚本放在：

- `home/users/<user>/scripts/`

用户脚本打包逻辑在：

- `home/users/<user>/scripts.nix`

对于 `mcbnixos`，Noctalia 的用户级入口还在：

- `home/users/mcbnixos/noctalia.nix`

目前的实际状态是：

- Home Manager 默认仍然打包 Shell 版用户脚本
- `scripts-rs/` 中已经提供对应 Rust 实现，适合独立运行、测试和后续迁移

这两条路线现在是并存关系，不是“Rust 已经完全接管 Home Manager 用户脚本打包链”。

这点在写文档和维护时必须说清楚，否则很容易以为所有用户脚本都已经换成 Rust 入口了。

---

## 7. `run.sh` 现在不是一坨脚本了

部署脚本已经拆层，主要分成：

- `scripts/run/cmd/`
  - 命令入口，例如部署和 release
- `scripts/run/lib/ui.sh`
  - 日志、菜单、确认、进度条
- `scripts/run/lib/env.sh`
  - 环境检查、脚本自检
- `scripts/run/lib/targets/`
  - 主机、用户、覆盖项收集
- `scripts/run/lib/pipeline.sh`
  - 源码准备、同步、重建、DNS 重试
- `scripts/run/lib/wizard.sh`
  - 交互式向导主流程与摘要

这次拆分很重要，因为它改变了维护方式：

- 以前你改部署流程，容易在一个长脚本里到处跳
- 现在你至少能按职责改对应分层文件

---

## 8. `scripts-rs` 不是摆设，它现在能做到什么

`scripts-rs/` 里是这套脚本体系的 Rust 实现。

比较关键的事实：

- `run-rs` 已具备完整部署 / release 流程实现
- `noctalia-gpu-mode-rs` 已具备完整 GPU 模式状态、菜单与应用逻辑
- 其他 `*-rs` 也覆盖了现有大部分用户脚本

但目前仓库默认入口仍然是：

- `./run.sh`

也就是说，Rust 路线已经能用，但仓库主流程还没有强制切过去。
这是一种比较稳的过渡方式：先把能力补齐，再决定什么时候把默认入口切换。

---

## 9. Zed 和 YesPlayMusic 为什么单独打包

它们现在都走自定义包目录：

- `pkgs/zed/`
- `pkgs/yesplaymusic/`

目的不是为了“炫技”，而是为了控制两个东西：

- 上游版本
- 固定 hash

这样你可以更主动地追官网稳定版，而不是完全被 nixpkgs 的节奏牵着走。

相关更新脚本：

- `pkgs/zed/scripts/update-source.sh`
- `pkgs/yesplaymusic/scripts/update-source.sh`
- `pkgs/scripts/update-upstream-apps.sh`

如果你平时想统一追新，这个入口最省事：

```bash
./pkgs/scripts/update-upstream-apps.sh
```

---

## 10. 图形运行时补丁，为什么存在

桌面环境里总会碰到一类程序：

- 不是从 nixpkgs 来的二进制
- 或者是上游自己打包的应用
- 或者是 rustup / cargo / AppImage 这类运行时比较野的东西

它们常见的问题是：

- Vulkan ICD 找不到
- `LD_LIBRARY_PATH` 不完整
- 图形栈依赖继承得不稳定

这套仓库把这块集中放在：

- `mcb.desktop.graphicsRuntime.*`

这样做的好处是，你不用每次为某个上游二进制单独想一套临时补丁，而是可以在主机级统一收口。

---

## 11. 维护时最值得坚持的习惯

如果你想让这套仓库长期还能舒服维护，最值钱的不是多会写 Nix，而是下面这几个习惯：

- 改系统层之前，先确认这是不是用户层问题
- 改主机层之前，先确认这是不是 profile 应该处理的默认值
- 加软件之前，先问自己这是“共享能力”还是“个人需要”
- 动大结构之前，先跑 `nix flake check`
- 遇到奇怪行为时，先确认当前改动跨了几层

很多“项目越来越重”的根本原因，不是功能太多，而是边界开始模糊。

这套仓库现在最重要的价值，不只是功能齐，而是边界还算清楚。
维护的时候，尽量把这点保住。
