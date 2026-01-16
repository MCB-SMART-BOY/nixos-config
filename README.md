# 🌸 NixOS Configuration

一套面向日常使用与开发的 NixOS 25.11 配置，采用 **Flake + Home Manager** 构建，结构清晰、可复用、便于扩展。

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
- **Shell**：Zsh + Oh-My-Zsh + Starship
- **编辑器**：Helix + 完整 LSP
- **状态栏/通知**：Waybar + Mako
- **启动器**：Fuzzel
- **主题**：Catppuccin Mocha
- **输入法**：fcitx5 + rime

## 🚀 快速开始

### 1) 初次部署

```bash
# 克隆仓库
git clone <your-repo-url> nixos-config
cd nixos-config

# 部署前自检（含网络可达性检查）
./run.sh preflight

# 同步硬件配置（必须；若用 scripts/install.sh 可自动同步）
sudo cp /etc/nixos/hardware-configuration.nix ./hardware-configuration.nix

# 可选：根据实际用户/代理/TUN 调整
$EDITOR host.nix

# 使用脚本部署
chmod +x run.sh scripts/*.sh
./run.sh

# 或分步执行
./run.sh preflight
./run.sh install

# 或直接使用 flake
sudo nixos-rebuild switch --flake .#nixos
```

> scripts/install.sh 默认会同步仓库到 `/etc/nixos`，可用 `--no-sync-etc` 关闭。
> 如果缺少 `hardware-configuration.nix`，构建会失败。

#### scripts/install.sh 常用参数

```bash
./scripts/install.sh --yes                    # 跳过确认
./scripts/install.sh --mode test             # 使用 nixos-rebuild test
./scripts/install.sh --show-trace            # 打印完整堆栈
./scripts/install.sh --force-sync            # 覆盖已有硬件配置
./scripts/install.sh --no-sync                # 跳过硬件配置同步
./scripts/install.sh --no-sync-etc            # 不同步仓库到 /etc/nixos
./scripts/install.sh --no-rebuild             # 仅同步不重建
./scripts/install.sh --skip-preflight         # 跳过部署前检查
./scripts/install.sh --skip-toolchain         # 跳过工具链安装
./scripts/install.sh --temp-dns               # 临时 DNS（默认 223.5.5.5 223.6.6.6 1.1.1.1 8.8.8.8）
./scripts/install.sh --dns 223.5.5.5 --dns 1.1.1.1
```

#### scripts/install_from_github.sh（云端同步）

```bash
./scripts/install_from_github.sh \
  --repo https://github.com/MCB-SMART-BOY/nixos-config.git \
  --branch master
```

或通过统一入口：

```bash
./run.sh cloud
./run.sh install_from_github --repo https://github.com/MCB-SMART-BOY/nixos-config.git --branch master
```

说明：
- 默认保留本机 `/etc/nixos/hardware-configuration.nix`，如需覆盖请加 `--force-hardware`
- 执行 `nixos-rebuild` 后会由 Home Manager 生成并链接 `~/.config` 配置
- 如需跳过自检可使用 `--skip-preflight`
- 如需临时 DNS 可使用 `--temp-dns` 或多次传入 `--dns`
- 默认会安装开发工具链（rustup），可用 `--skip-toolchain` 关闭

一行下载到本地：

```bash
curl -fsSL -o install_from_github.sh https://github.com/MCB-SMART-BOY/nixos-config/releases/latest/download/install_from_github.sh
chmod +x install_from_github.sh
```

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

也可以使用脚本：

```bash
./run.sh flake_update
./run.sh rebuild --mode switch
```

## 🧭 结构概览

```
nixos-config/
├── run.sh                    # 统一脚本入口
├── flake.nix                  # Flake 入口
├── flake.lock                 # 版本锁定（可复现）
├── host.nix                   # 主机入口（单主机）
├── hardware-configuration.nix # 硬件配置
├── modules/                   # 系统模块（default.nix 聚合）
├── home/                      # Home Manager 用户入口
│   ├── home.nix               # 入口模块
│   ├── modules/               # 子模块拆分
│   ├── config/                # 应用配置文件
│   ├── assets/                # 资源文件（壁纸等）
│   └── scripts/               # 用户侧脚本
├── configuration.nix          # 非 Flake 兼容入口
├── scripts/                   # 部署脚本
│   ├── README.md              # 脚本说明
│   ├── install.sh             # 本地部署
│   ├── install_from_github.sh # 云端同步部署
│   ├── preflight.sh           # 部署前自检
│   ├── sync_etc.sh            # 同步到 /etc/nixos
│   ├── sync_hardware.sh       # 同步硬件配置
│   ├── rebuild.sh             # nixos-rebuild 封装
│   ├── flake_update.sh        # flake.lock 更新
│   ├── home_refresh.sh        # Home Manager 刷新
│   ├── status.sh              # 状态查看
│   ├── doctor.sh              # 综合检查
│   ├── clean.sh               # Nix 垃圾回收
│   └── lib.sh                 # 公共函数
├── docs/                      # 说明文档
└── README.md
```

## ⚙️ 核心配置入口

### 系统层（NixOS）

- 主机入口：`host.nix`
- 入口：`modules/default.nix`
- 网络/代理：`modules/networking.nix`、`modules/services.nix`
- 字体/输入法/桌面：`modules/fonts.nix`、`modules/i18n.nix`、`modules/desktop.nix`

### 用户层（Home Manager）

- 入口：`home/home.nix`
- 应用配置：`home/config/*`
- 具体模块：`home/modules/*.nix`

### 主机变量

- `host.nix`：用户名、代理地址、TUN 网卡名等统一入口

## 🧩 包组开关

用户层包组可按需开关，位置：`home/modules/packages.nix`

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

## 🖥️ 桌面与自启动

Waybar / mako / swaybg / swayidle / fcitx5 由 **niri 的 spawn-at-startup** 管理：

- 编辑 `home/config/niri/config.kdl` 的 `spawn-at-startup`
- 壁纸由 `wallpaper-random` 登录时随机设置（目录：`~/Pictures/Wallpapers`）

## 🧰 日常维护

- 修改主机配置：编辑 `host.nix`
- 修改用户名：更新 `host.nix` 与 `home/` 路径
- 跨机器部署：调整 `host.nix` 中 `vars.user`、`vars.proxyUrl`、`vars.tunInterface`，并同步硬件配置
- 常用脚本入口：`./run.sh list`、`./run.sh status`、`./run.sh doctor`
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

要纳入仓库管理的壁纸，请放入 `home/assets/wallpapers` 后重建。

### 修改显示器配置

编辑 `home/config/niri/config.kdl`，调整 output 段落。

### 添加更多 LSP

1. 在 `home/config/helix/languages.toml` 添加语言配置
2. 在 `home/modules/packages.nix` 添加对应 LSP 包

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
