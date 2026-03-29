# NixOS 配置，不只是把系统“堆起来”

这套仓库的目标很直接：把一台会长期使用的 NixOS 主机，整理成一份能复用、能扩展、也能让人读得下去的配置。

它不是“只适合作者自己”的私人快照，也不是那种看起来模块很多、真正要改时却无从下手的样板工程。这里把系统层、主机层、用户层、脚本层都拆开了，你可以清楚知道自己正在改哪一层，以及这次改动会影响谁。

如果你第一次来到这个仓库，最重要的结论先说在前面：

- 想把系统先跑起来，直接用 `./run.sh`
- 想按用户区分软件，不要把东西都塞进系统层，去写 `home/users/<user>/packages.nix`
- 想搞清楚目录怎么分工，先看 `docs/STRUCTURE.md`
- 想知道平时怎么维护，先看 `docs/USAGE.md`

---

## 这套配置适合谁

- 你有不止一台 NixOS 主机，想统一管理
- 你在同一台机器上有多用户需求，不想所有人吃同一份桌面软件清单
- 你希望系统层和用户层边界清楚，不想以后越来越乱
- 你需要代理、TUN、GPU specialisation、Noctalia 这类“日常真的会用到”的东西

---

## 你会在这里得到什么

- Flake + Home Manager 的分层结构
- 多主机、多用户的统一组织方式
- 用户级软件逐个声明，而不是全局硬塞
- 桌面主机与服务器主机的不同 profile
- Niri + Wayland 的桌面路线
- Zed 官网稳定版与 YesPlayMusic 官网稳定版的固定打包与追新脚本
- `run.sh` 的交互式部署流程
- `scripts-rs/` 中一套对应的 Rust 脚本实现
- `flake check` 下的脚本语法检查、ShellCheck 和 `scripts-rs` 的 `cargo check`

---

## 如果你现在就要开始

### 1. 克隆仓库

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config
```

### 2. 直接跑部署向导

```bash
./run.sh
```

`run.sh` 现在是全交互模式。它会一步一步问你：

- 你是在“新增/调整用户”，还是“只更新当前配置”
- 你要用本地仓库、远端固定版本，还是远端最新版本
- 目标主机是谁
- 这台主机有哪些用户、哪些人有管理员权限
- 是否启用 per-user TUN
- 是否要做 GPU 覆盖
- 如果是服务器，要不要打开 Docker / Libvirt / CLI 工具组

你不需要在第一次上手时就把所有 Nix 文件摸透。先用向导把系统部署起来，再回头精修，是这套仓库更推荐的节奏。

### 3. 日常更新

```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

如果你只是想先试一遍、不急着切换：

```bash
sudo nixos-rebuild test --flake .#<hostname>
```

---

## 这套仓库最重要的约定

### 系统共享的东西，放系统层

比如：

- 基础运行时
- 网络 CLI / GUI
- Wayland 基础工具
- 系统级服务
- 桌面图形运行时

这些内容主要由 `hosts/profiles/*.nix` 和 `modules/*.nix` 控制。

### 只属于某个用户的软件，放用户层

比如：

- Zed
- YesPlayMusic
- 浏览器、聊天软件、办公软件
- 某个用户自己才需要的开发工具

这些内容不要再往 `environment.systemPackages` 里塞，而是写在：

- `home/users/<user>/packages.nix`

这样做的好处很实际：

- 不同用户的软件声明互不干扰
- Nix store 仍然共享构建产物，不会重复安装一份又一份
- 你以后看配置时，能一眼看出“这是系统共有”还是“这是某个用户自己要的”

---

## 目录先别全记，先记这几个入口

系统层：

- `hosts/<hostname>/default.nix`
- `hosts/profiles/desktop.nix`
- `hosts/profiles/server.nix`
- `modules/`

用户层：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/config/`

脚本层：

- `run.sh`
- `scripts/run/`
- `scripts-rs/`
- `home/users/<user>/scripts/`

如果你只想快速理解目录分工，不要在 README 里硬读完整个树，直接去看：

- [docs/STRUCTURE.md](/home/mcbgaruda/projects/nixos-config/docs/STRUCTURE.md)

---

## 关于 `run.sh` 和 `scripts-rs`

现在仓库里有两条脚本路线：

- `run.sh`
  - 当前默认部署入口
  - 交互式向导完整可用
  - 已经拆成 `scripts/run/cmd` 和 `scripts/run/lib`
- `scripts-rs/`
  - 对应的一套 Rust 实现
  - `run-rs` 与 `noctalia-gpu-mode-rs` 已不再委托 Bash
  - 适合你希望把脚本能力逐步迁到 Rust 时使用和维护

如果你只是部署系统，文档里仍然优先写 `./run.sh`。
如果你在维护脚本体系，`scripts-rs/README.md` 会更适合你。

---

## 你大概率会改到哪里

- 改主机名、默认用户、管理员用户：`hosts/<hostname>/default.nix`
- 改系统共享包组：`hosts/profiles/*.nix` 和 `modules/packages.nix`
- 给某个用户加软件：`home/users/<user>/packages.nix`
- 放某个用户自己的私有覆盖：`home/users/<user>/local.nix`
- 改 Niri / Noctalia / 主题配置：`home/users/<user>/config/`
- 改代理 / TUN / 路由：`modules/networking.nix` 和主机配置
- 改 GPU specialisation：`modules/hardware/gpu.nix` 和主机配置

---

## GPU、代理、多用户这些功能在这里是认真的

这套仓库不是只把软件装起来就结束了，它对下面这些问题是有明确组织方式的：

- GPU 模式：`igpu` / `hybrid` / `dgpu`
- Noctalia 顶栏切换 GPU specialisation
- `mcb.proxyMode = "tun" | "http" | "off"`
- per-user TUN 与按用户分流
- 多用户软件声明与共享 Nix store

如果你现在正卡在这些地方，不要直接在仓库里到处搜字符串，先看对应文档：

- GPU / 日常维护：[docs/USAGE.md](/home/mcbgaruda/projects/nixos-config/docs/USAGE.md)
- 结构分工：[docs/STRUCTURE.md](/home/mcbgaruda/projects/nixos-config/docs/STRUCTURE.md)
- 深层联动关系：[docs/DETAILS.md](/home/mcbgaruda/projects/nixos-config/docs/DETAILS.md)
- 国内网络与代理排障：[docs/NETWORK_CN.md](/home/mcbgaruda/projects/nixos-config/docs/NETWORK_CN.md)

---

## 文档索引

- [docs/USAGE.md](/home/mcbgaruda/projects/nixos-config/docs/USAGE.md)
  - 从零部署、日常更新、加用户、改 GPU、常见维护流程
- [docs/STRUCTURE.md](/home/mcbgaruda/projects/nixos-config/docs/STRUCTURE.md)
  - 仓库目录图和“改什么去哪里”
- [docs/DETAILS.md](/home/mcbgaruda/projects/nixos-config/docs/DETAILS.md)
  - 模块联动、参数含义、脚本路线、包和桌面细节
- [docs/NETWORK_CN.md](/home/mcbgaruda/projects/nixos-config/docs/NETWORK_CN.md)
  - 中国大陆网络环境下的下载、镜像、DNS、代理与 TUN 排障
- [scripts-rs/README.md](/home/mcbgaruda/projects/nixos-config/scripts-rs/README.md)
  - Rust 脚本集合的构建、使用与定位

---

## 最后一个建议

第一次接手这套配置时，不要急着“全面理解”。
更有效的方式是：

1. 先把系统跑起来
2. 再只追你眼前那一个问题
3. 每次只弄清楚一层配置为什么存在

这样这套仓库会越来越顺手，而不是越来越重。
