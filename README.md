# 🌸 NixOS Configuration

一套面向日常使用与开发的 NixOS 配置（Flake 使用 nixos-unstable，legacy 入口固定 25.11），采用 **Flake + Home Manager** 构建，结构清晰、可复用、便于扩展。

> 适合希望用模块化方式管理系统与用户环境的人，默认走 Niri + Wayland 路线。

## 📌 目录

- [✨ 亮点](#-亮点)
- [🚀 快速开始](#-快速开始)
- [🧭 结构概览](#-结构概览)
- [⚙️ 核心配置入口](#️-核心配置入口)
- [🧩 包组开关](#-包组开关)
- [🖥️ 桌面与自启动](#️-桌面与自启动)
- [🧰 日常维护](#-日常维护)
- [⌨️ 快捷键速查](#️-快捷键速查)
- [🎨 自定义](#-自定义)
- [🧯 故障排除](#-故障排除)
- [📚 参考资源](#-参考资源)

## ✨ 亮点

- **窗口管理器**：niri（Wayland 平铺、平滑滚动）
- **结构组织**：Flake + Home Manager 模块化分层
- **Shell**：Zsh + Oh-My-Zsh + Starship + fastfetch
- **编辑器**：Helix + 完整 LSP
- **状态栏/通知**：Waybar + Mako
- **启动器**：Fuzzel
- **监控**：btop（Noctalia 主题）
- **主题**：Catppuccin Mocha（GTK）+ Noctalia（终端/Waybar）
- **输入法**：fcitx5 + rime

## 🚀 快速开始

### 1) 一键部署（推荐）

```bash
curl -fsSL -o run.sh https://raw.githubusercontent.com/MCB-SMART-BOY/nixos-config/master/run.sh
chmod +x run.sh
./run.sh
```

说明：
- 默认从 GitHub 拉取最新代码并同步到 `/etc/nixos`
- 拉取顺序：Gitee 优先，其次 GitHub
- 如遇拉取或重建失败，会临时切换阿里云 DNS（223.5.5.5/223.6.6.6）后重试
- 默认执行 `nixos-rebuild switch --show-trace --upgrade`
- 默认保留本机硬件配置（`hardware-configuration.nix` 或 `hosts/<hostname>/hardware-configuration.nix`）
- 默认主机 `nixos`，默认用户 `mcbnixos`，默认覆盖 `/etc/nixos`
- 可使用 `--host` / `--user` / `--users` / `--backup` / `--overwrite` / `--ask`

### 2) 日常更新

```bash
sudo nixos-rebuild switch --flake .#nixos
sudo nixos-rebuild test   --flake .#nixos
sudo nixos-rebuild build  --flake .#nixos
```

### 3) 更新依赖版本

```bash
nix flake update
sudo nixos-rebuild switch --flake .#nixos
```


## 🧭 结构概览

```
nixos-config/
├── run.sh                    # 一键部署脚本
├── flake.nix                  # Flake 入口
├── flake.lock                 # 版本锁定（可复现）
├── hosts/                     # 主机配置目录
│   ├── profiles/              # 主机配置组合
│   ├── laptop/                # 笔记本模板
│   └── server/                # 服务器模板
├── modules/                   # 系统模块（default.nix 聚合）
├── home/                      # Home Manager 用户入口
│   ├── profiles/              # 用户配置组合
│   │   ├── full.nix
│   │   └── minimal.nix        # 精简 profile（服务器用）
│   ├── modules/               # 子模块拆分
│   └── users/                 # 用户入口（私有配置）
│       └── <user>/            # 用户目录
│           ├── config/         # 用户应用配置
│           ├── assets/         # 用户资源文件
│           └── scripts/        # 用户侧脚本
├── configuration.nix          # 非 Flake 兼容入口
├── docs/                      # 说明文档
└── README.md
```

## ⚙️ 核心配置入口

### 系统层（NixOS）

- 主机入口：`hosts/<hostname>/default.nix`
- 主机 Profiles：`hosts/profiles/desktop.nix` / `hosts/profiles/server.nix`
  - 服务器建议搭配用户 `home/profiles/minimal.nix`
- 网络/代理：`modules/networking.nix`、`modules/services.nix`
- 字体/输入法/桌面：`modules/fonts.nix`、`modules/i18n.nix`、`modules/desktop.nix`

### 用户层（Home Manager）

- 入口：`home/users/<user>/default.nix`
  - 用户专属配置放在 `home/users/<user>/`（如 git 身份、files.nix）
- 应用配置：`home/users/<user>/config/*`
- 具体模块：`home/modules/*.nix`

### 主机变量

- `hosts/<hostname>/default.nix`：用户名、代理地址、TUN 网卡名、CPU 类型、代理开关等统一入口
- 多用户时请设置 `mcb.users = [ "user1" "user2" ];`

## 🧩 包组开关

系统包组可按需开关，定义在 `modules/packages.nix`，建议在 `hosts/profiles/*.nix` 中设置 `mcb.packages`。

```nix
mcb.packages.enableGaming = false;
mcb.packages.enableEntertainment = false;
mcb.packages.enableGeekTools = false;
```

开关说明（按功能分组）：
- enableNetwork：代理/网络工具
- enableShellTools：终端与基础 CLI 工具
- enableWaylandTools：Wayland 桌面组件
- enableBrowsersAndMedia：浏览器/媒体/文件管理
- enableDev：开发工具链与 LSP
- enableChat：社交聊天
- enableEmulation：Wine/兼容层
- enableEntertainment：影音/阅读
- enableGaming：游戏相关
- enableSystemTools：系统维护工具
- enableTheming：主题与外观
- enableXorgCompat：Xwayland 兼容
- enableGeekTools：调试/诊断/极客工具

系统层游戏开关（NixOS）：

```nix
mcb.system.enableGaming = false;
```

## 🖥️ 桌面与自启动

Waybar / mako / swaybg / swayidle / fcitx5 由 **niri 的 spawn-at-startup** 管理：

- 编辑 `home/users/<user>/config/niri/config.kdl` 的 `spawn-at-startup`
- 壁纸由 `wallpaper-random` 登录时随机设置（目录：`~/Pictures/Wallpapers`）
- Waybar 自定义模块脚本位于 `home/users/<user>/scripts/waybar-*`，会安装到 `~/.local/bin/`

## 🧰 日常维护

- 修改主机配置：编辑 `hosts/<hostname>/default.nix`
- 修改用户名：更新 `hosts/<hostname>/default.nix` 与 `home/users/<user>/` 路径
- 跨机器部署：调整 `hosts/<hostname>/default.nix` 中 `mcb.user`、`mcb.proxyMode`、`mcb.proxyUrl`、`mcb.enableProxyDns`、`mcb.proxyDnsAddr`、`mcb.proxyDnsPort`、`mcb.tunInterface`、`mcb.perUserTun`、`mcb.cpuVendor`，并同步硬件配置
- 新增主机：在 `hosts/` 新建目录并放置 `default.nix`，flake 会自动发现
- 多用户：新增 `home/users/<user>/default.nix`，并把用户加到 `mcb.users`
- 传统非 Flake 入口：

```bash
sudo cp configuration.nix /etc/nixos/configuration.nix
sudo nixos-rebuild switch
```
> `configuration.nix` 会联网拉取 Home Manager（首次构建需要网络）

## ⌨️ 快捷键速查

### niri 窗口管理

| 快捷键 | 功能 |
|--------|------|
| `Mod+Return` | 打开终端 |
| `Mod+Space` | 应用启动器 |
| `Mod+Q` | 关闭窗口 |
| `Mod+H/J/K/L` | 焦点移动 |
| `Mod+Shift+H/J/K/L` | 窗口移动 |
| `Mod+1-9` | 切换工作区 |
| `Mod+Shift+1-9` | 移动到工作区 |
| `Mod+F` | 最大化列 |
| `Mod+Shift+F` | 全屏 |
| `Mod+C` | 居中列 |
| `Mod+R` | 切换预设宽度 |
| `Mod+E` | 文件管理器 |
| `Mod+B` | 浏览器 |
| `Print` | 截图 |

### Helix 编辑器

| 快捷键 | 功能 |
|--------|------|
| `Space+f` | 文件选择器 |
| `Space+b` | 缓冲区选择器 |
| `Space+s` | 符号选择器 |
| `Space+a` | 代码操作 |
| `Space+r` | 重命名 |
| `gd` | 跳转定义 |
| `gr` | 查找引用 |
| `gi` | 跳转实现 |
| `Ctrl+/` | 切换注释 |
| `jk` | 退出插入模式 |

## 🎨 自定义

### 更换壁纸

默认在登录时从 `~/Pictures/Wallpapers` 随机选择一张。

```bash
wallpaper-random
```

要纳入仓库管理的壁纸，请放入 `home/users/<user>/assets/wallpapers` 后重建。

### 修改显示器配置

编辑 `home/users/<user>/config/niri/config.kdl`，调整 output 段落。

### Fastfetch / btop 美化

- fastfetch：`home/users/<user>/config/fastfetch/config.jsonc`
- btop 配置：`home/users/<user>/config/btop/btop.conf`
- btop 主题：`home/users/<user>/config/btop/themes/noctalia.theme`

### 添加更多 LSP

1. 在 `home/users/<user>/config/helix/languages.toml` 添加语言配置
2. 在 `modules/packages.nix` 添加对应 LSP 包

## 🧯 故障排除

- niri 无法启动：
  ```bash
  journalctl --user -u niri -f
  ```

- Waybar 异常：
  ```bash
  pkill waybar && waybar &
  ```

- 输入法异常：
  ```bash
  pkill fcitx5 && fcitx5 -d -r
  ```

- 输入法无拼音选项：
  ```bash
  fcitx5-configtool
  ```
  确认已安装 `fcitx5-chinese-addons`，并在输入法列表中添加 Pinyin/Rime 后重启。

- 网络问题：参见 `docs/NETWORK_CN.md`

## 📚 参考资源

- [NixOS Manual](https://nixos.org/manual/nixos/stable/)
- [niri Wiki](https://github.com/YaLTeR/niri/wiki)
- [Helix Documentation](https://docs.helix-editor.com/)
- [Catppuccin Theme](https://catppuccin.com/)

## 📄 更多文档

- 结构说明：`docs/STRUCTURE.md`
- 项目细节：`docs/DETAILS.md`
- 国内网络：`docs/NETWORK_CN.md`

---

Made with ❤️ for a clean NixOS workflow
