# NixOS 配置，目标是长期可维护

这套仓库不是把一台机器先堆起来再说，而是尽量把边界讲清楚：

- 哪些东西属于整台机器
- 哪些东西只属于某个用户
- 哪些能力应该放系统层
- 哪些行为应该交给用户层

如果你第一次打开这个仓库，先记住三件事：

- 部署入口现在是 `nix run .#run-rs`
- 某个用户自己的软件，去写 `home/users/<user>/packages.nix`
- 用户脚本入口不再是散落的 Shell 文件，而是 `scripts-rs/` + `home/users/<user>/scripts.nix`

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
nix run .#run-rs
```

`run-rs` 现在就是完整的交互式部署向导。它会帮你处理：

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
- `modules/`

用户层：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/config/`
- `home/users/<user>/scripts.nix`

脚本与打包：

- `scripts-rs/`
- `pkgs/scripts-rs/`
- `pkgs/zed/`
- `pkgs/yesplaymusic/`

## 这次仓库已经变了什么

现在仓库里的脚本路线已经统一到 Rust：

- 部署与 release：`run-rs`
- Noctalia / 用户命令：`scripts-rs/src/bin/*.rs`
- 官网应用追新：`update-zed-source-rs`、`update-yesplaymusic-source-rs`、`update-upstream-apps-rs`
- 系统服务里的包装逻辑：也已经改成调用 Rust 二进制

换句话说，仓库里不再保留 `run.sh` 和那套分层 Shell 脚本作为主实现。

## 你大概率会改到哪里

- 改主机名、默认用户、管理员用户：`hosts/<hostname>/default.nix`
- 改系统共享包组：`hosts/profiles/*.nix` 和 `modules/packages.nix`
- 改某个用户的软件：`home/users/<user>/packages.nix`
- 改用户界面配置：`home/users/<user>/config/`
- 改 Noctalia / 桌面行为：`home/modules/desktop.nix`、`home/users/<user>/scripts.nix`
- 改代理 / TUN / 路由：`modules/networking.nix` 与 `modules/services/core.nix`
- 改 GPU specialisation：`modules/hardware/gpu.nix`
- 改脚本工具本身：`scripts-rs/src/bin/*.rs` 与 `scripts-rs/src/lib.rs`

## 文档索引

- [docs/USAGE.md](/home/mcbgaruda/projects/nixos-config/docs/USAGE.md)
  日常部署、更新、加用户、追新
- [docs/STRUCTURE.md](/home/mcbgaruda/projects/nixos-config/docs/STRUCTURE.md)
  目录分工与改动入口
- [docs/DETAILS.md](/home/mcbgaruda/projects/nixos-config/docs/DETAILS.md)
  多用户、GPU、Noctalia、脚本接线这些联动关系
- [docs/NETWORK_CN.md](/home/mcbgaruda/projects/nixos-config/docs/NETWORK_CN.md)
  国内网络环境下的下载、DNS、代理与 TUN 排障
- [scripts-rs/README.md](/home/mcbgaruda/projects/nixos-config/scripts-rs/README.md)
  Rust 脚本集合怎么构建、怎么接进整个仓库

## 最后一个建议

不要试图第一次就把所有目录都看懂。

更有效的节奏通常是：

1. 先把系统跑起来。
2. 只追眼前这个问题。
3. 每次只把一层边界看明白。

这套仓库就是按这种维护方式设计的。
