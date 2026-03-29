# 项目结构说明

如果你打开仓库后的第一反应是“我到底该改哪一层”，这页就是给这个时刻准备的。

这套仓库现在的组织原则很简单：

- `hosts/` 和 `modules/` 管机器
- `home/users/` 管人
- `scripts-rs/` 管脚本逻辑
- `pkgs/` 管仓库自己维护的包

## 一眼先记住这些入口

- 主机入口：`hosts/<hostname>/default.nix`
- 系统模块：`modules/`
- 用户入口：`home/users/<user>/default.nix`
- 用户软件：`home/users/<user>/packages.nix`
- 用户脚本接线：`home/users/<user>/scripts.nix`
- Rust 脚本集合：`scripts-rs/`
- Rust 脚本打包：`pkgs/scripts-rs/`

如果你只记住这些位置，已经足够处理大部分维护工作。

## 顶层目录怎么理解

```text
.
├── flake.nix
├── flake.lock
├── configuration.nix
├── scripts-rs/
├── hosts/
├── modules/
├── home/
├── pkgs/
├── docs/
└── README.md
```

### `hosts/`

这里回答的是：“这台机器是谁？”

常见内容：

- `hosts/<hostname>/default.nix`
  主机入口，决定这台机器导入哪个 profile、默认用户是谁、有哪些主机级覆盖
- `hosts/<hostname>/system.nix`
  机器架构，例如 `"x86_64-linux"`
- `hosts/<hostname>/hardware-configuration.nix`
  这台机器自己的硬件配置
- `hosts/<hostname>/local.nix`
  主机私有覆盖

### `hosts/profiles/`

这里回答的是：“这台机器大体属于哪类角色？”

例如：

- `hosts/profiles/desktop.nix`
- `hosts/profiles/server.nix`

它们是组合好的主机模板，不是某一台具体机器。

### `modules/`

这里放系统层公共能力。

例如：

- 用户与权限
- 网络与代理
- GPU
- 服务
- 系统共享包组

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

### `home/users/<user>/`

这是你最常改的地方之一。

常见文件：

- `default.nix`
  用户入口
- `packages.nix`
  这个用户的软件声明
- `scripts.nix`
  用户命令如何从 `scripts-rs` 包接到 `~/.local/bin`
- `local.nix`
  不想进仓库的私有覆盖
- `local.nix.example`
  起步示例
- `config/`
  会被链接到 `~/.config`
- `assets/`
  这个用户自己的资源文件

这里最重要的变化是：

- 用户命令现在不再从 `home/users/<user>/scripts/` 读取原始 Shell 脚本
- 而是通过 `scripts-rs` 编译出来的二进制来提供

### `scripts-rs/`

这里放 Rust 写的脚本实现。

常见内容：

- `scripts-rs/src/bin/*.rs`
  一个文件对应一个命令
- `scripts-rs/src/lib.rs`
  公共函数和复用逻辑

这里现在不只是“备用路线”，而是仓库的正式脚本实现。

### `pkgs/`

这里放仓库自己维护的包和包装逻辑。

现在比较关键的是：

- `pkgs/scripts-rs/`
- `pkgs/zed/`
- `pkgs/yesplaymusic/`

如果你想追官网稳定版，或者把仓库内部工具做成 Nix 包，这里就是核心位置。

## 按“我要改什么”来定位

### 我要改主机名、默认用户、管理员用户

看：

- `hosts/<hostname>/default.nix`

### 我要改系统级共享软件或服务

看：

- `hosts/profiles/*.nix`
- `modules/packages.nix`
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

- `home/users/<user>/scripts.nix`

### 我要改部署或追新工具

看：

- `scripts-rs/src/bin/run-rs.rs`
- `scripts-rs/src/bin/update-zed-source-rs.rs`
- `scripts-rs/src/bin/update-yesplaymusic-source-rs.rs`
- `scripts-rs/src/bin/update-upstream-apps-rs.rs`

## 最值得坚持的边界

一句话总结：

- 机器共享能力放系统层
- 用户个性化声明放用户层

只要这个边界不乱，仓库再大也还能读。
一旦这个边界开始混，后面每次改东西都会越来越累。
