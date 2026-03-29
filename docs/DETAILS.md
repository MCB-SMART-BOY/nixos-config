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
- 改用户命令接线：`home/users/<user>/scripts.nix`
- 改部署流程：`scripts-rs/src/bin/run-rs.rs`
- 改 Rust 公共逻辑：`scripts-rs/src/lib.rs`

## 1. Home Manager 这一层，负责“人”

`home/` 这一层回答的是：

“某个用户登录之后，他看到的终端、编辑器、桌面栏、应用清单，应该是什么样？”

关键入口：

- `home/users/<user>/default.nix`
- `home/profiles/full.nix`
- `home/profiles/minimal.nix`
- `home/modules/*.nix`

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

其中最容易踩坑的是 `hybrid`：

- 需要正确的 busId
- 需要 BIOS / MUX 支持
- 需要理解当前机器是不是已经被锁成 `dGPU-only`

`run-rs` 在向导里会优先尝试自动探测 busId，探测不到才回退到主机现有配置。

## 6. Noctalia 与用户命令，现在是怎么接起来的

这一块以前最容易被误解成“几个零碎脚本”，现在其实已经很清楚了：

- Rust 命令实现放在 `scripts-rs/src/bin/*.rs`
- Nix 打包入口在 `pkgs/scripts-rs/default.nix`
- 用户侧链接入口在 `home/users/<user>/scripts.nix`

也就是说，当前状态已经不是：

- 某个用户目录里堆一堆原始 Shell 脚本

而是：

- 仓库统一维护一套 Rust 二进制
- 用户侧决定要不要把哪些命令链接进自己的环境

这也是为什么你现在去找 `home/users/<user>/scripts/`，已经不应该再找到真实实现了。

## 7. 部署流程现在怎么理解

部署流程的主入口现在是：

- `scripts-rs/src/bin/run-rs.rs`

它负责的事情包括：

- 环境检查
- 源码准备
- 仓库自检
- 目标主机与用户收集
- `/etc/nixos` 同步
- `nixos-rebuild` 重建
- release 发布

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

- `update-zed-source-rs`
- `update-yesplaymusic-source-rs`
- `update-upstream-apps-rs`

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
- 改脚本逻辑时，先看 `scripts-rs/src/lib.rs` 里有没有现成复用函数
- 改 Noctalia / 桌面按钮时，不要只看配置文件，也要看命令入口是不是已经接好

这几个习惯会直接决定仓库是越来越清楚，还是越改越粘。
