# 项目结构说明

本项目采用标准 Flake 布局，区分系统层（NixOS）与用户层（Home Manager），并保留 `configuration.nix` 作为非 Flake 兼容入口。主机相关内容统一放入 `hosts/` 目录，方便后续扩展多主机。

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
│   └── <hostname>/
├── run.sh
├── README.md
├── docs/
├── modules/
└── home/
```

## 目录职责

- `hosts/<hostname>/default.nix`：主机入口，聚合系统模块并导入硬件配置
- `hosts/<hostname>/hardware-configuration.nix`：由 `nixos-generate-config` 生成（可选）
- `hosts/<hostname>/local.nix`：本机覆盖配置（可选，`run.sh` 可生成）
- `hosts/profiles/`：主机配置组合（如 desktop/server）

- `modules/`：系统级模块集合（网络、安全、字体、服务等）

- `home/`
  - `home/users/<user>/default.nix`：Home Manager 用户入口
  - `home/profiles/`：用户配置组合（profile）
  - `home/modules/`：用户模块拆分
  - `home/users/<user>/config/`：用户应用配置（链接到 ~/.config，含 waybar/foot/fuzzel/fastfetch/btop）
  - `home/users/<user>/assets/`：用户资源文件（例如壁纸）
  - `home/users/<user>/scripts/`：用户侧脚本（例如随机壁纸、Waybar 模块脚本）
  - `home/users/<user>/`：用户专属配置（如 git 身份）

- `run.sh`：一键部署脚本（拉取 GitHub → 同步 `/etc/nixos` → `nixos-rebuild`）

## 关键入口

- Flake：`flake.nix`
- NixOS 入口：`hosts/<hostname>/default.nix`
- Home Manager 入口：`home/users/<user>/default.nix`
- 兼容入口：`configuration.nix`
