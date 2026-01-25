# 项目结构说明

本项目采用标准 Flake 布局，区分系统层（NixOS）与用户层（Home Manager），并保留 `configuration.nix` 作为非 Flake 兼容入口。为了单用户、单主机场景，主机入口与硬件配置直接放在仓库根目录。

## 顶层结构

```
.
├── flake.nix
├── flake.lock
├── configuration.nix
├── host.nix
├── hardware-configuration.nix
├── run.sh
├── README.md
├── docs/
├── modules/
└── home/
```

## 目录职责

- `host.nix`：主机入口，定义 vars，聚合系统模块并导入硬件配置
- `hardware-configuration.nix`：由 `nixos-generate-config` 生成

- `modules/`：系统级模块集合（网络、安全、字体、服务等）

- `home/`
  - `home/home.nix`：Home Manager 用户入口
  - `home/modules/`：用户模块拆分
  - `home/config/`：应用配置文件（由 Home Manager 链接到 ~/.config，含 waybar/foot/fuzzel/fastfetch/btop）
  - `home/assets/`：资源文件（例如壁纸）
  - `home/scripts/`：用户侧脚本（例如随机壁纸、Waybar 模块脚本）

- `run.sh`：一键部署脚本（拉取 GitHub → 同步 `/etc/nixos` → `nixos-rebuild`）

## 关键入口

- Flake：`flake.nix`
- NixOS 入口：`host.nix`
- Home Manager 入口：`home/home.nix`
- 兼容入口：`configuration.nix`
