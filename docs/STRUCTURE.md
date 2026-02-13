# 项目结构说明

本项目采用标准 Flake 布局，区分系统层（NixOS）与用户层（Home Manager），并保留 `configuration.nix` 作为非 Flake 兼容入口。主机相关内容统一放入 `hosts/` 目录，便于多主机扩展。

## 快速定位

- 主机入口：`hosts/<hostname>/default.nix`
- 系统模块：`modules/`
- 用户入口：`home/users/<user>/default.nix`
- 用户配置：`home/users/<user>/config/`
- 使用说明书：`docs/USAGE.md`

## 顶层结构

```
.
├── flake.nix
├── flake.lock
├── configuration.nix
├── hosts/
│   ├── profiles/
│   ├── laptop/
│   ├── server/
│   └── nixos/
├── run.sh
├── README.md
├── docs/
├── modules/
└── home/
```

## 目录职责

- `hosts/<hostname>/default.nix`
  - 主机入口，聚合系统模块并导入硬件配置
- `hosts/<hostname>/hardware-configuration.nix`
  - `nixos-generate-config` 生成（可选）
- `hosts/<hostname>/local.nix`
  - 本机覆盖配置（可选，`run.sh` 可生成）
- `hosts/<hostname>/system.nix`
  - 必填系统架构字符串（如 `"x86_64-linux"` / `"aarch64-linux"`）
- `hosts/profiles/`
  - 主机配置组合（desktop/server）

- `modules/`
  - 系统级模块（网络、安全、字体、服务等）

- `home/`
  - `home/users/<user>/default.nix`：用户入口
  - `home/profiles/`：用户配置组合
  - `home/modules/`：用户模块拆分
  - `home/users/<user>/config/`：应用配置（链接到 ~/.config）
  - `home/users/<user>/assets/`：用户资源（如壁纸）
  - `home/users/<user>/scripts/`：用户脚本（如 Waybar 模块脚本）

- `run.sh`
  - 一键部署脚本（拉取 → 同步 → `nixos-rebuild`）

## 关键入口

- Flake：`flake.nix`
- NixOS 入口：`hosts/<hostname>/default.nix`
- Home Manager 入口：`home/users/<user>/default.nix`
- Legacy 入口：`configuration.nix`
