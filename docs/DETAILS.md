# 项目细节说明

该文档用于说明本仓库的关键配置点与联动关系，便于维护与扩展。

## Home Manager 体系

- 用户入口：`home/home.nix`
- 典型模块：
  - `home/modules/base.nix`：环境变量与 PATH
  - `home/modules/packages.nix`：包组控制开关（`mcb.packages.*`）
  - `home/modules/programs.nix`：Alacritty/Helix 等程序配置接入
  - `home/modules/desktop.nix`：niri/fuzzel/mako/waybar/swaylock/gtk 配置接入
  - `home/modules/shell.nix`：zsh/direnv/zoxide/starship 接入
  - `home/modules/git.nix`：git 基础配置

配置文件分离在 `home/config/`，由 `xdg.configFile` 统一链接到 `~/.config`。

## NixOS 体系

- 系统入口：`host.nix`
- 聚合模块：`modules/default.nix`
- 用户账号与临时目录规则：`host.nix`
- 主机变量：`host.nix` 中 `vars` 字段
  - `user`：用户名
  - `proxyUrl`：系统代理默认地址
  - `tunInterface`：TUN 网卡名（与相关服务配置一致）

## 部署脚本

- `install.sh`：部署脚本

常用逻辑：
- 同步 `/etc/nixos/hardware-configuration.nix` 到 `hardware-configuration.nix`
- 运行 `nixos-rebuild switch --flake .#nixos`

注意：
- 如需修改 flake 目标名，请同步更新 `install.sh` 中的 `TARGET_NAME`
- 可通过 `TARGET_NAME` 与 `MODE` 环境变量临时覆盖目标与模式

常用参数：
- `-y/--yes`：跳过确认
- `--mode switch|test|build`：指定 `nixos-rebuild` 模式
- `--show-trace`：启用完整堆栈
- `--force-sync`：覆盖现有硬件配置
- `--no-sync`：跳过硬件配置同步
- `--no-rebuild`：仅同步不重建

## 常见扩展方式

- 修改主机配置：编辑 `host.nix`
- 自定义应用配置：把文件放入 `home/config/` 后再通过模块接入
