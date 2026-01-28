# 项目细节说明

该文档用于说明本仓库的关键配置点与联动关系，便于维护与扩展。

## 快速定位（新手常见需求）

- **改默认用户/多用户**：`hosts/<hostname>/default.nix` -> `mcb.user` / `mcb.users`
- **改系统包组**：`hosts/profiles/desktop.nix`（开关） + `modules/packages.nix`（包列表）
- **改桌面快捷键**：`home/users/<user>/config/niri/config.kdl`
- **改应用主题**：`home/users/<user>/config/`（waybar/foot/alacritty/gtk）
- **改输入法**：`modules/i18n.nix` + `home/users/<user>/config/fcitx5/profile`

## Home Manager 体系

- 用户入口：`home/users/<user>/default.nix`
- Profiles：`home/profiles/full.nix` / `home/profiles/minimal.nix`
- 用户专属配置：`home/users/<user>/*.nix`（如 git 身份、私有文件映射）
- 典型模块：
  - `home/modules/base.nix`：环境变量与 PATH
  - `home/modules/programs.nix`：Alacritty/Helix 等程序启用
  - `home/modules/desktop.nix`：niri/fuzzel/mako/waybar/swaylock/gtk 功能启用
  - `home/users/<user>/scripts.nix`：用户脚本打包与安装（含 Waybar 自定义脚本）
  - `home/modules/shell.nix`：zsh/direnv/zoxide/starship/tmux/fastfetch/btop 启用
  - `home/modules/git.nix`：git 基础配置

配置文件分离在 `home/users/<user>/config/`，由 `xdg.configFile` 统一链接到 `~/.config`。
壁纸资源位于 `home/users/<user>/assets/wallpapers`，会链接到 `~/Pictures/Wallpapers`。
随机壁纸脚本位于 `home/users/<user>/scripts/wallpaper-random`，会安装到 `~/.local/bin/wallpaper-random`。
fastfetch 配置位于 `home/users/<user>/config/fastfetch/config.jsonc`。
btop 配置位于 `home/users/<user>/config/btop/btop.conf`，主题位于 `home/users/<user>/config/btop/themes/noctalia.theme`。
Waybar 自定义模块脚本位于 `home/users/<user>/scripts/waybar-*`，会安装到 `~/.local/bin/`。

## NixOS 体系

- 系统入口：`hosts/<hostname>/default.nix`
- 聚合模块：`modules/default.nix`（兼容用，可选）
- 选项定义：`modules/options.nix`（`mcb.*`）
- 系统包组：`modules/packages.nix`（`mcb.packages.*`）
- 主机 Profiles：`hosts/profiles/desktop.nix` / `hosts/profiles/server.nix`
- 用户账号与临时目录规则：`hosts/<hostname>/default.nix`
- 主机变量：`hosts/<hostname>/default.nix` 中 `mcb` 字段
  - `user`：用户名
  - `users`：同一主机需要启用的用户列表（Home Manager 会对每个用户生效）
  - `proxyMode`：代理模式（`tun` / `http` / `off`）
  - `proxyUrl`：系统 HTTP 代理地址（仅 `proxyMode = "http"` 生效）
  - `enableProxyDns`：TUN 模式下是否强制本地 DNS（默认开启）
  - `proxyDnsAddr`：本地 DNS 地址（默认 `127.0.0.1`）
  - `proxyDnsPort`：本地 DNS 端口（默认 `53`）
  - `tunInterface`：TUN 网卡名（与相关服务配置一致）
  - `tunInterfaces`：兼容多个 TUN 网卡名的列表（可选）
  - `cpuVendor`：CPU 类型（`intel` 或 `amd`）
  - `proxyMode = "tun"` 时使用本地 DNS，且不配置公网 fallback
  - `perUserTun.enable`：按用户 UID 路由的多实例 TUN
  - `perUserTun.interfaces`：用户 → TUN 网卡名（与各用户配置一致）
  - `perUserTun.redirectDns`：按用户 UID 重定向 DNS 到本地端口
  - `perUserTun.dnsPorts`：用户 → DNS 端口（需与各用户 Clash DNS listen 保持一致）
  - `perUserTun.tableBase`：路由表起始编号
  - `perUserTun.priorityBase`：ip rule 起始优先级
  - 启用 `perUserTun` 时需将 `enableProxyDns = false`
  - per-user 模式会生成 `clash-verge-service@<user>` 与 `mcb-tun-route@<user>` 服务

## 部署脚本

- `run.sh`：一键部署（支持选择主机/用户、备份或覆盖 `/etc/nixos`，默认先拉取 Gitee 后 GitHub，并执行 `nixos-rebuild switch --show-trace --upgrade`）
- 若使用 `run.sh` 指定用户，会在 `hosts/<hostname>/local.nix` 写入覆盖配置
- 如遇拉取或重建失败，会临时切换阿里云 DNS（223.5.5.5/223.6.6.6）后重试
- 默认保留本机硬件配置（`hardware-configuration.nix` 或 `hosts/<hostname>/hardware-configuration.nix`）
- `configuration.nix` 会联网拉取 Home Manager（首次构建需要网络）

## 常见扩展方式

- 修改主机配置：编辑 `hosts/<hostname>/default.nix`
- 自定义应用配置：把文件放入 `home/users/<user>/config/` 后在 `home/users/<user>/files.nix` 中接入
- 关闭系统层游戏功能：`mcb.system.enableGaming = false;`
