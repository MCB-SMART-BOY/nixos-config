# 🌸 NixOS Configuration

一套精美的 NixOS 25.11 配置，专为 Rust 开发者打造。

## ✨ 特性

- **窗口管理器**: [niri](https://github.com/YaLTeR/niri) - 现代化的可滚动平铺 Wayland 合成器
- **登录管理器**: greetd + tuigreet - 优雅的 TUI 登录界面
- **Shell**: Zsh + Oh-My-Zsh + Starship prompt
- **编辑器**: Helix - 后现代文本编辑器，完整 LSP 配置
- **终端**: Alacritty - GPU 加速终端
- **启动器**: Fuzzel - 快速应用启动器
- **状态栏**: Waybar - 高度可定制状态栏
- **主题**: Catppuccin Mocha 🎨
- **输入法**: fcitx5 + rime 中文输入

## 📁 文件结构

```
nixos-config/
├── configuration.nix     # NixOS 主配置 (单文件)
├── dotfiles/             # 用户配置文件
│   ├── helix/           # Helix 编辑器配置
│   ├── niri/            # niri 窗口管理器配置  
│   ├── zsh/             # Zsh 配置
│   ├── starship/        # Starship prompt
│   ├── waybar/          # 状态栏
│   ├── alacritty/       # 终端
│   └── fuzzel/          # 启动器
├── install.sh           # 一键部署脚本
└── README.md
```

## 🚀 一键安装

```bash
# 解压
tar -xzf nixos-config.tar.gz
cd nixos-config

# 运行安装脚本
chmod +x install.sh
./install.sh
```

脚本会自动：
1. 检查环境
2. 备份现有配置
3. 部署系统和用户配置
4. 重建 NixOS
5. 提示重启

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

### Zsh 别名

```bash
# Git
g    → git
ga   → git add
gc   → git commit
gp   → git push
gl   → git pull
lg   → lazygit

# Cargo
c    → cargo
cb   → cargo build
cr   → cargo run
ct   → cargo test
cc   → cargo check

# NixOS
nrs  → sudo nixos-rebuild switch
nsp  → nix search nixpkgs
```

## 🎨 自定义

### 更换壁纸

```bash
# 静态壁纸
swaybg -i ~/.config/wallpaper.jpg

# 动态壁纸 (GIF)
swww init && swww img ~/Pictures/animated.gif

# 视频壁纸
mpvpaper '*' ~/Videos/wallpaper.mp4 --fork

# Wallpaper Engine 壁纸 (需先在 Steam 安装 Wallpaper Engine)
linux-wallpaperengine --screen-root eDP-1 <workshop_id>
```

### 修改显示器设置

编辑 `~/.config/niri/config.kdl`，取消注释 output 部分并调整参数。

### 添加更多 LSP

编辑 `~/.config/helix/languages.toml`，参考已有配置添加新语言。

## 📺 动漫/漫画应用

### Suwayomi (Mihon/Tachiyomi 桌面版)

已通过 `services.suwayomi-server` 启用，启动后访问：
```
http://localhost:4567
```

### Kazumi (动漫流媒体)

通过 Flatpak 安装：
```bash
# 首次使用需添加 Flathub 仓库
flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

# 安装 Kazumi
flatpak install flathub io.github.Predidit.Kazumi

# 运行
flatpak run io.github.Predidit.Kazumi
```

### Mangayomi (漫画/动漫)

下载 AppImage 后直接运行：
```bash
chmod +x Mangayomi-*.AppImage
./Mangayomi-*.AppImage
# 或使用 appimage-run
appimage-run Mangayomi-*.AppImage
```

## 🔧 故障排除

### niri 无法启动

```bash
# 检查 niri 日志
journalctl --user -u niri -f
```

### Waybar 显示异常

```bash
# 重启 waybar
pkill waybar && waybar &
```

### 字体图标不显示

确保安装了 Nerd Fonts：

```bash
# 检查字体
fc-list | grep -i nerd
```

## 📚 参考资源

- [NixOS Manual](https://nixos.org/manual/nixos/stable/)
- [niri Wiki](https://github.com/YaLTeR/niri/wiki)
- [Helix Documentation](https://docs.helix-editor.com/)
- [Catppuccin Theme](https://catppuccin.com/)

---

Made with 💜 for Rust developers
