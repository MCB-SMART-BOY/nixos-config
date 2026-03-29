# 项目结构说明

如果你经常打开一个 NixOS 仓库，然后第一反应是“我到底该改哪一层”，这页就是给这种时候准备的。

这套仓库的结构不是为了显得模块很多，而是为了回答三个很实际的问题：

- 这是哪台机器的配置
- 这是系统共享的能力，还是某个用户自己的东西
- 这次改动会影响全部用户，还是只影响一个人

---

## 一眼先记住的入口

- 主机入口：`hosts/<hostname>/default.nix`
- 系统模块：`modules/`
- 用户入口：`home/users/<user>/default.nix`
- 用户软件：`home/users/<user>/packages.nix`
- 部署入口：`run.sh`
- Rust 脚本集合：`scripts-rs/`

如果你只记得这六个位置，已经足够处理绝大多数维护工作。

---

## 顶层目录，不用死背，先按职责理解

```text
.
├── flake.nix
├── flake.lock
├── configuration.nix
├── run.sh
├── scripts/
├── scripts-rs/
├── hosts/
├── modules/
├── home/
├── pkgs/
├── docs/
└── README.md
```

你可以把它们理解成下面这几类：

### `hosts/`

这里决定“这台机器是谁”。

每个子目录是一台主机，例如：

- `hosts/nixos/`
- `hosts/laptop/`
- `hosts/server/`

常见文件：

- `hosts/<hostname>/default.nix`
  - 主机入口，决定这台机器导入哪个 profile、默认用户是谁、有哪些主机级覆盖
- `hosts/<hostname>/system.nix`
  - 机器架构，例如 `"x86_64-linux"`
- `hosts/<hostname>/hardware-configuration.nix`
  - 这台机器自己的硬件配置
- `hosts/<hostname>/local.nix`
  - 主机私有覆盖，适合放“只属于这台机器”的补丁或临时调整

### `hosts/profiles/`

这里决定“这台机器大体是什么类型”。

目前最重要的是：

- `hosts/profiles/desktop.nix`
- `hosts/profiles/server.nix`

它们不是具体主机，而是组合好的主机模板。
你可以把它理解为“桌面机器默认该带哪些系统能力”，“服务器默认该关掉哪些图形部分”。

### `modules/`

这里放系统层模块，也就是所有主机共享的系统级能力。

例如：

- 网络
- 服务
- 字体
- 安全
- 用户与权限
- GPU
- 包组

如果一个改动应该影响一整类主机，而不是某个单独用户，通常就是去 `modules/` 或 `hosts/profiles/`。

### `home/`

这里放 Home Manager，也就是“用户怎么使用这台机器”。

常见结构：

- `home/profiles/`
  - 用户配置组合，例如完整桌面用户、最小服务器用户
- `home/modules/`
  - 用户层公共模块
- `home/users/<user>/`
  - 某个具体用户的入口与私有配置

这部分是本仓库很重要的边界线：

- 机器共享能力放系统层
- 个人软件与个性化设置放用户层

### `home/users/<user>/`

这是你最常改的目录之一。

你可以把它理解成“某个人自己的家目录蓝图”。

常见文件：

- `default.nix`
  - 用户入口
- `packages.nix`
  - 这个用户的软件声明
- `local.nix`
  - 这个用户本地私有覆盖，不想进仓库的东西可以放这里
- `local.nix.example`
  - 给你起步用的示例
- `config/`
  - 会被链接到 `~/.config`
- `assets/`
  - 该用户用到的资源文件
- `scripts/`
  - 用户脚本，例如 Noctalia 顶栏按钮脚本

如果你只是想“给某个用户加个软件”，不要去找系统层，直接改：

- `home/users/<user>/packages.nix`

### `pkgs/`

这里放仓库自己维护的包，以及这些包的上游版本 pin。

当前比较关键的是：

- `pkgs/zed/`
- `pkgs/yesplaymusic/`
- `pkgs/scripts/update-upstream-apps.sh`

当你想追官网稳定版，而不是单纯等 nixpkgs 更新时，这里就是核心位置。

### `scripts/`

这里放 Shell 路线的脚本。

当前最重要的是：

- `run.sh`
- `scripts/run/cmd/`
- `scripts/run/lib/`

也就是部署脚本的拆分版本。现在 `run.sh` 本身更像总入口，具体逻辑被拆到这些目录里了。

### `scripts-rs/`

这里放 Rust 路线的脚本实现。

它不是装饰性的“未来规划”，而是已经能工作的另一套实现，尤其是：

- `run-rs`
- `noctalia-gpu-mode-rs`

不过仓库当前默认部署文档仍然优先写 `./run.sh`，因为这仍是你最直接、最稳的入口。

### `docs/`

这里只有一个目的：减少你下一次重新理解项目的成本。

建议读法：

- 想上手：`docs/USAGE.md`
- 想看目录：`docs/STRUCTURE.md`
- 想看联动关系：`docs/DETAILS.md`
- 想排网络问题：`docs/NETWORK_CN.md`

---

## 按“我要改什么”来定位

### 我要改主机名、默认用户、管理员用户

看：

- `hosts/<hostname>/default.nix`

### 我要改系统级共享软件或服务

看：

- `hosts/profiles/*.nix`
- `modules/packages.nix`
- `modules/services*.nix`

### 我要给某个用户加软件

看：

- `home/users/<user>/packages.nix`

### 我要改 Niri / Noctalia / 终端 / 编辑器配置

看：

- `home/users/<user>/config/`

### 我要改用户脚本或顶栏行为

看：

- `home/users/<user>/scripts/`
- `home/users/<user>/scripts.nix`

### 我要改部署流程

看：

- `run.sh`
- `scripts/run/cmd/`
- `scripts/run/lib/`

### 我要改 Rust 版脚本

看：

- `scripts-rs/src/bin/*.rs`
- `scripts-rs/src/lib.rs`

---

## 这个仓库最重要的边界

一句话总结：

- `hosts/` 和 `modules/` 管机器
- `home/users/` 管人

只要这个边界不乱，仓库再大也还能维护。
一旦这个边界开始混，后面每次改东西都会越来越痛苦。

所以如果你拿不准某个配置该写在哪里，可以先问自己一句：

“这是这台机器都需要，还是某个用户自己需要？”

大多数时候，答案会直接把你带到正确目录。
