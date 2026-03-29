# NixOS 配置，目标是长期可维护

这套仓库不是把一台机器先堆起来再说，而是尽量把边界讲清楚：

- 哪些东西属于整台机器
- 哪些东西只属于某个用户
- 哪些能力应该放系统层
- 哪些行为应该交给用户层

如果你第一次打开这个仓库，先记住三件事：

- 控制台入口现在是 `nix run .#mcbctl`
- 显式 TUI 别名仍然是 `nix run .#mcb-tui`
- 如果你要直接进部署向导，使用 `nix run .#mcb-deploy`
- 某个用户自己的软件，去写 `home/users/<user>/packages.nix`
- 如果你想用 TUI 勾选软件并写入用户机器管理层，落点是 `home/users/<user>/managed/packages/*.nix`（由 `managed/packages.nix` 聚合）
- 用户命令不再靠用户目录里的脚本桥接，而是由 `mcbctl/` 编译并通过 `pkgs/mcbctl/` 进入环境

## 这套仓库适合什么场景

- 你有多台 NixOS 主机，想统一维护
- 同一台机器上有多用户需求，不想所有人共用一份桌面软件清单
- 你想把系统层和用户层彻底分开
- 你确实会用到 GPU specialisation、代理、per-user TUN、Noctalia 这些东西

## 如果你现在就想部署

先把仓库拉下来，然后在仓库根目录运行：

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config
nix run .#mcbctl
```

`mcbctl` 现在默认进入 TUI 控制台。你可以在里面继续走部署、用户、软件和后续设置管理。

如果你就是想直接进入旧式的交互式部署向导，可以运行：

```bash
nix run .#mcb-deploy
```

`mcb-deploy` 会帮你处理：

- 部署模式选择
- 本地仓库 / 远端固定版本 / 远端最新版本
- 主机选择
- 用户与管理员用户
- per-user TUN
- GPU 覆盖
- 服务器预设

如果你已经很熟悉这套仓库，也可以直接：

```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

## 这套仓库最重要的约定

系统共享的能力放系统层，例如：

- 基础运行时
- 网络与代理服务
- Wayland / 桌面运行时
- GPU、虚拟化、系统工具

只属于某个用户的软件和界面放用户层，例如：

- Zed
- YesPlayMusic
- 浏览器、聊天、办公软件
- 某个用户自己的开发工具
- Noctalia / Niri / 终端 / 编辑器配置

所以你以后要给某个用户加软件，优先改的是：

- `home/users/<user>/packages.nix`

而不是继续把所有东西塞进 `environment.systemPackages`。

## 先记住这些入口就够了

系统层：

- `hosts/<hostname>/default.nix`
- `hosts/profiles/desktop.nix`
- `hosts/profiles/server.nix`
- `hosts/templates/`
- `modules/`

用户层：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/packages/`（复杂用户可继续拆成“一个软件组一个文件”）
- `home/users/<user>/config/`
- `home/users/<user>/managed/`
- `home/templates/users/`

脚本与打包：

- `mcbctl/`
- `pkgs/mcbctl/`
- `pkgs/zed/`
- `pkgs/yesplaymusic/`

机器管理区：

- `hosts/<host>/managed/`
- `home/users/<user>/managed/`
- `catalog/packages/`（本地覆盖层 / 仓库内自维护包元数据，不再作为主软件源）
- `catalog/groups.toml`
- `catalog/home-options.toml`

模板区：

- `hosts/templates/`
- `home/templates/users/`

## 这次仓库已经变了什么

现在仓库里的脚本路线已经统一到 Rust：

- 控制台入口：`mcbctl`
- 直接部署 / release：`mcb-deploy`
- Noctalia / 用户命令：`mcbctl/src/bin/*.rs`
- 官网应用追新：`update-zed-source`、`update-yesplaymusic-source`、`update-upstream-apps`
- 系统服务里的包装逻辑：也已经改成调用 Rust 二进制

换句话说，仓库里不再保留 `run.sh` 和那套分层 Shell 脚本作为主实现。

同时，模板也已经从真实配置命名空间里分离出来：

- `hosts/` 只放真实主机
- `home/users/` 只放真实启用用户
- 模板统一放进 `hosts/templates/` 和 `home/templates/users/`

## 你大概率会改到哪里

- 改主机名、默认用户、管理员用户：`hosts/<hostname>/default.nix`
- 改系统共享包组：`hosts/profiles/*.nix` 和 `modules/packages.nix`
- 改某个用户的软件：`home/users/<user>/packages.nix`
- 改用户界面配置：`home/users/<user>/config/`
- 改 Noctalia / 桌面行为：`home/modules/desktop.nix`、`pkgs/mcbctl/default.nix`
- 改代理 / TUN / 路由：`modules/networking.nix` 与 `modules/services/core.nix`
- 改 GPU specialisation：`modules/hardware/gpu.nix`
- 改脚本工具本身：`mcbctl/src/bin/*.rs` 与 `mcbctl/src/lib.rs`

## 文档索引

- [docs/USAGE.md](/home/mcbgaruda/projects/nixos-config/docs/USAGE.md)
  日常部署、更新、加用户、追新
- [docs/STRUCTURE.md](/home/mcbgaruda/projects/nixos-config/docs/STRUCTURE.md)
  目录分工与改动入口
- [docs/DETAILS.md](/home/mcbgaruda/projects/nixos-config/docs/DETAILS.md)
  多用户、GPU、Noctalia、脚本接线这些联动关系
- [docs/NETWORK_CN.md](/home/mcbgaruda/projects/nixos-config/docs/NETWORK_CN.md)
  国内网络环境下的下载、DNS、代理与 TUN 排障
- [mcbctl/README.md](/home/mcbgaruda/projects/nixos-config/mcbctl/README.md)
  Rust 脚本集合怎么构建、怎么接进整个仓库

## 最后一个建议

不要试图第一次就把所有目录都看懂。

更有效的节奏通常是：

1. 先把系统跑起来。
2. 只追眼前这个问题。
3. 每次只把一层边界看明白。

这套仓库就是按这种维护方式设计的。
