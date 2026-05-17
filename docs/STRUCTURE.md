# 项目结构说明

两层架构：

- `modules/` — 系统怎么工作（通用，可克隆到任何机器）
- `users/` — 人怎么用（个性化）

机器特定值（用户名、主机名、时区、功能开关）在项目根目录的 `local.nix`（gitignored）。
硬件配置（GPU busId、磁盘分区）在 `hardware-configuration.nix`（`nixos-generate-config` 生成，gitignored）。
新用户可参考 `local.nix.example`。

---

## 一眼记住的入口

- 系统模块：`modules/default.nix`
- 用户入口：`users/<user>/default.nix`
- 用户软件：`users/<user>/packages.nix`
- 部署入口：`run.sh`
- 机器身份：`local.nix`（gitignored，模板：`local.nix.example`）
- 硬件配置：`hardware-configuration.nix`（gitignored）

---

## 顶层目录

```text
.
├── flake.nix
├── configuration.nix            # 非 Flake 入口（兼容）
├── local.nix                    # 机器身份（gitignored）
├── local.nix.example            # 机器身份模板
├── hardware-configuration.nix   # 硬件（gitignored）
├── run.sh
├── modules/                     # 系统层：通用模块
│   ├── options.nix              #   所有 mcb.* 选项声明（集中管理）
│   ├── default.nix              #   聚合 + mkDefault 默认值
│   └── ...                      #   功能模块
├── users/                       # 用户层：Home Manager
│   └── admin/                   #   唯一的预置用户
├── pkgs/                        # 自维护包
├── overlays/                    # nixpkgs 覆盖层
├── scripts/run/                 # 部署脚本
├── secrets/                     # sops-nix 机密
└── docs/
```

---

## `modules/` — 系统层

系统级模块。所有选项声明集中在 `options.nix`，各模块只读取 `config.mcb.*`。

| 文件 | 职责 |
|---|---|
| `default.nix` | 聚合所有子模块 + mkDefault 默认值 |
| `options.nix` | **所有** mcb.* 选项声明（用户/代理/包组/Flatpak/虚拟化/游戏/图形） |
| `users.nix` | 根据 mcb.users 自动创建系统账户 |
| `boot.nix` | 内核、sysctl、systemd-boot |
| `networking.nix` | NetworkManager、防火墙、DNS（48 行） |
| `nix.nix` | flakes、二进制缓存、GC |
| `security.nix` | polkit + sudo 权限策略 |
| `desktop.nix` | niri、greetd、Wayland 环境变量、xdg-portal |
| `gaming.nix` | Steam、gamemode（使用官方选项，local.nix 中以 mkForce 覆盖） |
| `virtualization.nix` | Docker / libvirt（使用官方选项，local.nix 中以 mkForce 覆盖） |
| `packages.nix` | 系统包组定义与拼接（选项在 options.nix） |
| `i18n.nix` | locale、fcitx5 输入法 |
| `fonts.nix` | 字体包 + fontconfig |
| `services/core.nix` | SSH、nix-ld（90 行） |
| `services/desktop.nix` | Flatpak / PipeWire / 蓝牙 / AppImage |

---

## `users/` — 用户层

每个子目录是一个用户。`admin` 是唯一的预置用户，其他用户通过 `./run.sh add-user` 生成。

```
users/admin/
├── default.nix      # 入口
├── packages.nix     # 软件清单（逐包注释）
├── git.nix          # Git 配置（自包含：options + config + 身份）
├── desktop.nix      # Noctalia / GPU wrapper / 桌面应用（bash 提取到 scripts/）
├── shell.nix        # Zsh / Starship / Tmux / direnv / zoxide
├── programs.nix     # Alacritty / Helix
├── base.nix         # XDG / PATH / 编辑器默认值
├── noctalia.nix     # Noctalia 顶栏个性化
├── files.nix        # dotfiles 映射
├── scripts.nix      # 用户脚本打包（Noctalia 按钮脚本）
├── config/          # dotfiles（niri、helix、fastfetch …）
└── scripts/         # 用户脚本源文件（含 GPU wrapper）
```

---

## `local.nix` — 机器身份（gitignored）

存放每台机器不同的值，用 `lib.mkForce` 覆盖 `modules/default.nix` 的默认值：

```nix
{ lib, ... }: {
  mcb.user = lib.mkForce "admin";
  mcb.users = lib.mkForce [ "admin" ];
  networking.hostName = lib.mkForce "nixos";
  time.timeZone = lib.mkForce "Asia/Shanghai";
}
```

新用户可复制 `local.nix.example` 开始。

---

## 改什么去哪

| 想改的 | 去 |
|---|---|
| 主机名、用户名、时区 | `local.nix` |
| GPU / 磁盘 | `hardware-configuration.nix` |
| 防火墙 / DNS | `modules/networking.nix` |
| 系统包组 | `modules/packages.nix` |
| 内核参数 | `modules/boot.nix` |
| 某个用户的软件 | `users/<user>/packages.nix` |
| dotfiles | `users/<user>/config/` |
| 部署流程 | `run.sh` + `scripts/run/` |
| Nix 缓存/GC | `modules/nix.nix` |
| 所有开关定义 | `modules/options.nix` |

---

## 核心边界

- `modules/` 管机器（通用，不写死值；选项在 `options.nix` 集中声明）
- `users/` 管人（个性化）
- `local.nix` 管"这台机器是谁"（gitignored）
- `hardware-configuration.nix` 管硬件（nixos-generate-config 生成，gitignored）
