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
├── home/
└── scripts/
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

- `scripts/`：部署脚本目录
  - `scripts/install.sh`：本地部署脚本
  - `scripts/install_from_github.sh`：云端同步部署脚本
  - `scripts/preflight.sh`：部署前自检脚本
  - `scripts/sync_etc.sh`：同步仓库到 `/etc/nixos`
  - `scripts/sync_hardware.sh`：同步硬件配置
  - `scripts/rebuild.sh`：封装 `nixos-rebuild`
  - `scripts/flake_update.sh`：更新 `flake.lock`
  - `scripts/home_refresh.sh`：刷新 Home Manager 服务
  - `scripts/status.sh`：快速状态查看
  - `scripts/doctor.sh`：综合检查
  - `scripts/clean.sh`：Nix 垃圾回收
  - `scripts/README.md`：脚本体系说明

## 关键入口

- Flake：`flake.nix`
- NixOS 入口：`host.nix`
- Home Manager 入口：`home/home.nix`
- 兼容入口：`configuration.nix`
