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
  - `tunInterfaces`：兼容多个 TUN 网卡名的列表（可选）
  - `enableProxy`：代理/TUN 相关服务与防火墙开关
  - `cpuVendor`：CPU 类型（`intel` 或 `amd`）
  - `proxyUrl` 为空时不启用系统代理与本地 DNS

## 部署脚本

- `run.sh`：一键部署（默认拉取 GitHub 最新代码、同步到 `/etc/nixos`、执行 `nixos-rebuild switch --show-trace --upgrade`）
- 默认保留本机 `/etc/nixos/hardware-configuration.nix`，如需覆盖请加 `--force-hardware`
- 可选参数：`--repo`、`--branch`、`--target`、`--mode`、`--aliyun-dns`、`--dns-iface`
- `configuration.nix` 会联网拉取 Home Manager（首次构建需要网络）

## 常见扩展方式

- 修改主机配置：编辑 `host.nix`
- 自定义应用配置：把文件放入 `home/config/` 后再通过模块接入
- 关闭系统层游戏功能：`mcb.system.enableGaming = false;`
