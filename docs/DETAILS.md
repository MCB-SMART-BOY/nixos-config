# 项目细节说明

该文档用于说明本仓库的关键配置点与联动关系，便于维护与扩展。

## Home Manager 体系

- 用户入口：`home/home.nix`
- 典型模块：
  - `home/modules/base.nix`：环境变量与 PATH
  - `home/modules/packages.nix`：包组控制开关（`mcb.packages.*`，含 `enableGeekTools`）
  - `home/modules/programs.nix`：Alacritty/Helix 等程序配置接入
  - `home/modules/desktop.nix`：niri/fuzzel/mako/waybar/swaylock/gtk 配置接入
  - `home/modules/shell.nix`：zsh/direnv/zoxide/starship/tmux 接入
  - `home/modules/git.nix`：git 基础配置

配置文件分离在 `home/config/`，由 `xdg.configFile` 统一链接到 `~/.config`。
壁纸资源位于 `home/assets/wallpapers`，会链接到 `~/Pictures/Wallpapers`。
随机壁纸脚本位于 `home/scripts/wallpaper-random`，会安装到 `~/.local/bin/wallpaper-random`。

## NixOS 体系

- 系统入口：`host.nix`
- 聚合模块：`modules/default.nix`
- 用户账号与临时目录规则：`host.nix`
- 主机变量：`host.nix` 中 `vars` 字段
  - `user`：用户名
  - `proxyUrl`：系统代理默认地址
  - `tunInterface`：TUN 网卡名（与相关服务配置一致）
  - `proxyUrl` 为空时不启用系统代理与本地 DNS

## 部署脚本

- `scripts/install.sh`：本地部署脚本
- `scripts/install_from_github.sh`：云端同步部署脚本

常用逻辑：
- 同步 `/etc/nixos/hardware-configuration.nix` 到 `hardware-configuration.nix`
- 同步仓库配置到 `/etc/nixos`
- 运行 `nixos-rebuild switch --flake .#nixos`
- 重建后由 Home Manager 生成并链接 `~/.config`

注意：
- 如需修改 flake 目标名，请同步更新 `scripts/install.sh` 中的 `TARGET_NAME`
- `scripts/install_from_github.sh` 默认保留本机的 `hardware-configuration.nix`
- 可通过 `TARGET_NAME` 与 `MODE` 环境变量临时覆盖目标与模式
 - `configuration.nix` 会联网拉取 Home Manager（首次构建需要网络）

常用参数：
- `-y/--yes`：跳过确认
- `--mode switch|test|build`：指定 `nixos-rebuild` 模式
- `--show-trace`：启用完整堆栈
- `--force-sync`：覆盖现有硬件配置
- `--no-sync`：跳过硬件配置同步
- `--no-sync-etc`：不同步仓库到 `/etc/nixos`
- `--no-rebuild`：仅同步不重建

## 常见扩展方式

- 修改主机配置：编辑 `host.nix`
- 自定义应用配置：把文件放入 `home/config/` 后再通过模块接入
