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

# 同步硬件配置（必须；若用 install.sh 可自动同步）
sudo cp /etc/nixos/hardware-configuration.nix ./hosts/nixos-dev/hardware-configuration.nix

# 可选：根据实际用户/代理/TUN 调整
$EDITOR lib/vars.nix

# 使用脚本部署
chmod +x install.sh
./install.sh nixos-dev

# 或直接使用 flake
sudo nixos-rebuild switch --flake .#nixos-dev
```

> 如果缺少 `hosts/<host>/hardware-configuration.nix`，构建会失败。

#### install.sh 常用参数

```bash
./install.sh --yes                    # 跳过确认
./install.sh --no-sync                # 跳过硬件配置同步
./install.sh --no-rebuild             # 仅同步不重建
./install.sh --host <name>            # 指定主机名
./install.sh --init-host --host <name> # 基于模板初始化新主机
```

### 2) 日常更新

```bash
sudo nixos-rebuild switch --flake .#nixos-dev
sudo nixos-rebuild test   --flake .#nixos-dev
sudo nixos-rebuild build  --flake .#nixos-dev
```

### 3) 更新依赖版本

```bash
nix flake update
sudo nixos-rebuild switch --flake .#nixos-dev
```

## 🧭 结构概览

```
nixos-config/
├── flake.nix                  # Flake 入口
├── flake.lock                 # 版本锁定（可复现）
├── hosts/nixos-dev/           # 主机入口
│   ├── default.nix
│   └── hardware-configuration.nix
├── nixos/modules/             # 系统模块（default.nix 聚合）
├── lib/vars.nix               # 共享常量（用户名/代理/TUN）
├── home/users/mcbnixos/        # Home Manager 用户入口
│   ├── home.nix               # 入口模块
│   ├── modules/               # 子模块拆分
│   └── config/                # 应用配置文件
├── configuration.nix          # 非 Flake 兼容入口
├── scripts/install.sh         # 一键部署脚本（主脚本）
├── install.sh                 # 入口包装（转发到 scripts/）
└── README.md
```

## ⚙️ 核心配置入口

### 系统层（NixOS）

- 入口：`nixos/modules/default.nix`
- 网络/代理：`nixos/modules/networking.nix`、`nixos/modules/services.nix`
- 字体/输入法/桌面：`nixos/modules/fonts.nix`、`nixos/modules/i18n.nix`、`nixos/modules/desktop.nix`

### 用户层（Home Manager）

- 入口：`home/users/mcbnixos/home.nix`
- 应用配置：`home/users/mcbnixos/config/*`
- 具体模块：`home/users/mcbnixos/modules/*.nix`

### 共享常量

- `lib/vars.nix`：用户名、代理地址、TUN 网卡名等统一入口

## 🧩 包组开关

用户层包组可按需开关，位置：`home/users/mcbnixos/modules/packages.nix`

```nix
mcb.packages.enableGaming = false;
mcb.packages.enableEntertainment = false;
```

## 🖥️ 桌面与自启动

Waybar / mako / swaybg / swayidle / fcitx5 由 **niri 的 spawn-at-startup** 管理：

- 编辑 `home/users/mcbnixos/config/niri/config.kdl` 的 `spawn-at-startup`

## 🧰 日常维护

- 新增主机：复制 `hosts/nixos-dev` 为新目录，并在 `flake.nix` 注册
- 修改用户名：更新 `lib/vars.nix` 与 `home/users/<user>/` 路径
- 传统非 Flake 入口：

```bash
sudo cp configuration.nix /etc/nixos/configuration.nix
sudo nixos-rebuild switch
```

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

```bash
cp /path/to/wallpaper.jpg ~/.config/wallpaper.jpg
pkill swaybg && swaybg -i ~/.config/wallpaper.jpg -m fill &
```

### 修改显示器配置

编辑 `home/users/mcbnixos/config/niri/config.kdl`，调整 output 段落。

### 添加更多 LSP

1. 在 `home/users/mcbnixos/config/helix/languages.toml` 添加语言配置
2. 在 `home/users/mcbnixos/modules/packages.nix` 添加对应 LSP 包

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

- 网络问题：参见 `NETWORK_CN.md`

## 📚 参考资源

- [NixOS Manual](https://nixos.org/manual/nixos/stable/)
- [niri Wiki](https://github.com/YaLTeR/niri/wiki)
- [Helix Documentation](https://docs.helix-editor.com/)
- [Catppuccin Theme](https://catppuccin.com/)

---

Made with ❤️ for a clean NixOS workflow
