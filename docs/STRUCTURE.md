# 项目结构说明

两层架构：

- `modules/` — 系统怎么工作（通用，可克隆到任何机器）
- `users/` — 人怎么用（个性化）

机器特定值（GPU busId、用户名、主机名、时区）在项目根目录的 `local.nix`（gitignored）。硬件配置在 `hardware-configuration.nix`（`nixos-generate-config` 生成，gitignored）。

---

## 一眼记住的入口

- 系统模块：`modules/default.nix`
- 用户入口：`users/<user>/default.nix`
- 用户软件：`users/<user>/packages.nix`
- 部署入口：`run.sh`
- 机器身份：`local.nix`（gitignored）
- 硬件配置：`hardware-configuration.nix`（gitignored）

---

## 顶层目录

```text
.
├── flake.nix
├── local.nix                   # 机器身份（gitignored）
├── hardware-configuration.nix  # 硬件（gitignored）
├── run.sh
├── modules/                    # 系统层：通用模块
├── users/                      # 用户层：Home Manager
│   └── admin/                  #   唯一的预置用户
├── pkgs/                       # 自维护包
├── overlays/                   # nixpkgs 覆盖层
├── scripts/run/                # 部署脚本
├── secrets/                    # sops-nix 机密
└── docs/
```

---

## `modules/` — 系统层

系统级模块，描述"一台 NixOS 桌面该怎么工作"。不包含机器特定值。

| 文件 | 职责 |
|---|---|
| `default.nix` | 聚合所有子模块 + mkDefault 默认值 |
| `options.nix` | mcb.* 选项声明（user、users、proxyMode …） |
| `users.nix` | 根据 mcb.users 自动创建系统账户 |
| `boot.nix` | 内核、sysctl、systemd-boot |
| `networking.nix` | NetworkManager、防火墙、DNS |
| `nix.nix` | flakes、二进制缓存、GC |
| `security.nix` | polkit 权限策略 |
| `desktop.nix` | niri、greetd、Wayland 环境变量、xdg-portal |
| `gaming.nix` | Steam、gamemode、mangohud |
| `virtualization.nix` | Docker / libvirt 开关 |
| `packages.nix` | 系统包组（CLI/GUI/主题 …） |
| `i18n.nix` | locale、fcitx5 输入法 |
| `fonts.nix` | 字体包 + fontconfig |
| `services/core.nix` | SSH、nix-ld |
| `services/desktop.nix` | Flatpak 支持 |

---

## `users/` — 用户层

每个子目录是一个用户，包含该用户完整的 Home Manager 配置。`admin` 是唯一的预置用户，其他用户通过 `./run.sh add-user` 生成。

```
users/admin/
├── default.nix      # 入口
├── packages.nix     # 软件清单
├── git.nix          # Git 配置（自包含：options + config + 身份）
├── desktop.nix      # Noctalia / GPU wrapper / 桌面应用
├── shell.nix        # Zsh / Starship / Tmux / direnv / zoxide
├── programs.nix     # Alacritty / Helix
├── base.nix         # XDG / PATH / 编辑器默认值
├── noctalia.nix     # Noctalia 顶栏个性化
├── files.nix        # dotfiles 映射
├── scripts.nix      # 用户脚本打包
├── config/          # dotfiles（niri、helix、fastfetch …）
└── scripts/         # 用户脚本源文件
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

---

## 核心边界

- `modules/` 管机器（通用，不写死值）
- `users/` 管人（个性化）
- `local.nix` 管"这台机器是谁"（gitignored）
- `hardware-configuration.nix` 管硬件（nixos-generate-config 生成，gitignored）
