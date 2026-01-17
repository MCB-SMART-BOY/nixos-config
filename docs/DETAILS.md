# 项目细节说明

该文档用于说明本仓库的关键配置点与联动关系，便于维护与扩展。

## Home Manager 体系

- 用户入口：`home/home.nix`
- 典型模块：
  - `home/modules/base.nix`：环境变量与 PATH
  - `home/modules/packages.nix`：包组控制开关（`mcb.packages.*`，含 `enableGeekTools`、`enableHeavyBuilds`）
  - `home/modules/programs.nix`：Alacritty/Helix 等程序配置接入
  - `home/modules/desktop.nix`：niri/fuzzel/mako/waybar/swaylock/gtk 配置接入（含 Waybar 自定义脚本）
  - `home/modules/shell.nix`：zsh/direnv/zoxide/starship/tmux/fastfetch/btop 接入
  - `home/modules/git.nix`：git 基础配置

配置文件分离在 `home/config/`，由 `xdg.configFile` 统一链接到 `~/.config`。
壁纸资源位于 `home/assets/wallpapers`，会链接到 `~/Pictures/Wallpapers`。
随机壁纸脚本位于 `home/scripts/wallpaper-random`，会安装到 `~/.local/bin/wallpaper-random`。
fastfetch 配置位于 `home/config/fastfetch/config.jsonc`。
btop 配置位于 `home/config/btop/btop.conf`，主题位于 `home/config/btop/themes/noctalia.theme`。
Waybar 自定义模块脚本位于 `home/scripts/waybar-*`，会安装到 `~/.local/bin/`。

## NixOS 体系

- 系统入口：`host.nix`
- 聚合模块：`modules/default.nix`
- 用户账号与临时目录规则：`host.nix`
- 主机变量：`host.nix` 中 `vars` 字段
  - `user`：用户名
  - `proxyUrl`：系统代理默认地址
  - `tunInterface`：TUN 网卡名（与相关服务配置一致）
  - `enableProxy`：代理/TUN 相关服务与防火墙开关
  - `cpuVendor`：CPU 类型（`intel` 或 `amd`）
  - `proxyUrl` 为空时不启用系统代理与本地 DNS

## 部署脚本

- 统一入口：`run.sh`（默认 `./run.sh` = preflight + install，`./run.sh cloud` = GitHub 拉取）
- `scripts/install.sh`：本地部署脚本
- `scripts/install_from_github.sh`：云端同步部署脚本
- `scripts/preflight.sh`：部署前自检（含网络与关键依赖检查）
- `scripts/toolchain.sh`：安装开发工具链（rustup）
- `scripts/sync_etc.sh`：同步仓库到 `/etc/nixos`
- `scripts/sync_hardware.sh`：同步硬件配置
- `scripts/rebuild.sh`：封装 `nixos-rebuild`
- `scripts/flake_update.sh`：更新 `flake.lock`
- `scripts/home_refresh.sh`：刷新 Home Manager systemd 服务
- `scripts/status.sh`：快速状态查看
- `scripts/doctor.sh`：综合检查
- `scripts/clean.sh`：Nix 垃圾回收（默认 dry-run）
- `scripts/README.md`：脚本说明

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
- `--skip-preflight`：跳过部署前自检
- `--temp-dns`：部署期间临时指定 DNS
- `--dns <ip>`：追加临时 DNS（可多次）
- `--skip-toolchain`：跳过工具链安装

## 常见扩展方式

- 修改主机配置：编辑 `host.nix`
- 自定义应用配置：把文件放入 `home/config/` 后再通过模块接入
- 关闭系统层游戏功能：`mcb.system.enableGaming = false;`
