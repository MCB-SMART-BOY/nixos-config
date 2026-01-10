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
├── README.md
├── docs/
├── modules/
├── home/
└── install.sh
```

## 目录职责

- `host.nix`：主机入口，定义 vars，聚合系统模块并导入硬件配置
- `hardware-configuration.nix`：由 `nixos-generate-config` 生成

- `modules/`：系统级模块集合（网络、安全、字体、服务等）

- `home/`
  - `home/home.nix`：Home Manager 用户入口
  - `home/modules/`：用户模块拆分
  - `home/config/`：应用配置文件（由 Home Manager 链接到 ~/.config）

- `install.sh`：部署脚本

## 关键入口

- Flake：`flake.nix`
- NixOS 入口：`host.nix`
- Home Manager 入口：`home/home.nix`
- 兼容入口：`configuration.nix`
