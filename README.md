# NixOS Configuration

个人 NixOS 桌面工作站配置，使用 Flakes 管理。当前仓库包含一个名为 `nixos` 的 `x86_64-linux` 系统配置，基于锁定的 `nixpkgs/nixos-unstable`。

## 功能概览

- Niri、Hyprland 与 GNOME 桌面环境；GDM、Wayland、Xwayland、XDG Desktop Portal 和 Plymouth
- `fcitx5` 输入法，包含 Rime、GTK、Qt 和中文输入相关组件
- NVIDIA open kernel module、32 位图形支持及 NVIDIA Container Toolkit
- Podman、Docker 兼容命令/socket、Incus、libvirt/KVM、QEMU、virt-manager 和 virtiofsd
- Steam、GameMode、Lutris、MangoHud，以及 Steam Remote Play 防火墙规则
- PipeWire、蓝牙、Blueman、Flatpak、打印、AppImage 和电源管理
- AppArmor、auditd、polkit、sudo、GnuPG agent 和 zram swap
- 常用命令行工具、开发工具、Wayland 工具、字体及中文字体

## 快速开始

### 环境要求

- 一台运行 NixOS 的 `x86_64-linux` 主机
- 已启用 Flakes 和 `nix-command`
- 具备使用 `sudo nixos-rebuild` 的权限

### 检查与部署

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config

# 构建全部源码检查和当前平台的主机系统闭包
nix flake check -L --keep-going

# 应用 nixos 主机配置；构建阶段会再次依赖源码检查
sudo nixos-rebuild switch --flake .#nixos
```

`switch` 只有在格式、Statix、Deadnix、密钥扫描和系统闭包全部构建成功后才会进入激活阶段。检查 derivation 按 `source-format`、`source-statix`、`source-deadnix` 和 `source-secrets` 独立命名，失败日志会直接指出对应检查。

首次使用前必须检查并按目标主机修改：

- `machines/nixos/hardware-configuration.nix`：文件系统、swap 和硬件扫描结果
- `machines/nixos/hardware-gpu.nix`：GPU 驱动与硬件相关设置
- `machines/nixos/default.nix`：用户、用户组和主机专属设置
- `modules/network.nix`：主机名、NetworkManager 和 Clash Verge 设置

当前主机文件定义了 `mcbnixos` 普通用户，但没有在仓库中提供密码或 SSH authorized key。部署到自己的主机时，应通过安全的本地方式设置凭据，绝不要把密码或私钥写入仓库。

`hardware-configuration.nix` 必须保持为本机 `nixos-generate-config --show-hardware-config` 的原始输出，不承载手工维护配置，也不参与 formatter、Statix 或 Deadnix。当前生成结果包含 UEFI `/boot`、XFS 根分区、swap，以及运行中 Incus 产生的挂载项；这些条目来自生成器，不是在该文件中手工维护的虚拟化配置。其他虚拟化设置统一放在 `modules/virtualisation.nix`。

## 仓库结构

```text
.
├── flake.nix                     # Flake 入口与 nixpkgs 输入
├── flake/
│   ├── default.nix               # 只组合各职责并导出 Flake outputs
│   ├── machines.nix              # 主机发现、NixOS 配置与主机闭包检查
│   ├── source-checks.nix         # 源码选择与检查 derivation
│   └── development.nix           # formatter 与 devShell
├── scripts/
│   ├── run-source-check.sh       # 统一检查入口和失败语义
│   └── check-*.sh                # 各项检查的 Bash 实现
├── statix.toml                   # 项目 Statix 规则
├── machines/
│   └── nixos/
│       ├── default.nix           # 主机用户与主机专属设置
│       ├── system.nix            # 必需的主机系统架构
│       ├── hardware-configuration.nix
│       └── hardware-gpu.nix
└── modules/
    ├── default.nix               # 模块总入口
    ├── boot.nix                  # systemd-boot、内核
    ├── desktop.nix               # 桌面、显示管理器、Portal
    ├── i18n.nix                  # locale、时区、fcitx5
    ├── fonts.nix                 # 系统字体与中文字体
    ├── packages.nix              # 系统软件包
    ├── virtualisation.nix        # Podman、Incus、libvirt
    ├── game.nix                  # Steam、GameMode
    ├── applications.nix          # PipeWire、蓝牙、Flatpak 等
    ├── network.nix               # NetworkManager、Clash Verge
    ├── security.nix              # AppArmor、auditd、polkit、sudo
    ├── nix.nix                   # Nix 设置、系统构建检查、GC、zram swap
    ├── lib.nix                   # 公共 Nix 辅助函数
    └── core.nix                  # nix-ld 运行库
```

`flake/machines.nix` 只扫描一次 `machines/`，将包含 `default.nix` 的目录视为主机，并要求每台主机同时提供 `system.nix`；缺失时会在求值阶段报告主机名和所需路径。主机发现继续使用 `macDirs`、`macEts`、`macNms`、`macSys` 和 `mkMac` 这组项目既有命名。显式目标架构避免构建结果依赖执行命令的当前机器。

当前没有自定义 package override，因此不创建空的 `overlays/`。以后出现多个输出共同使用的 package override 时，应将其放入独立的 `overlays/` 目录，而不是混入主机或检查逻辑。

## 开发与验证

```bash
# 使用锁定 nixpkgs 提供的 formatter
nix fmt .

# 进入包含 nixfmt、Statix、Deadnix、Gitleaks、Trivy 和 Vulnix 的开发环境
nix develop

# 构建全部纯检查和当前平台的主机系统闭包
nix flake check -L --keep-going

# 网络审计：Trivy 扫描其支持的源码/制品，Vulnix 扫描 NixOS 闭包
nix develop -c trivy fs .
nix develop -c vulnix --closure ./result
```

纯构建检查覆盖以下范围：

- `format`：检查手工维护的 Nix 文件是否符合 `nixfmt`；生成的 `hardware-configuration.nix` 保持原样
- `statix`、`deadnix`：检查同一组手工维护代码；`statix.toml` 关闭 `empty_pattern` 以允许空参数模块使用 `{ ... }:`，并关闭无法按文件排除生成配置的 `repeated_keys`
- `secrets`：用 Gitleaks 扫描当前 Git Flake 源快照，并对日志脱敏
- `nixos-<主机名>`：构建对应主机的完整系统闭包

Git Flake 源快照不包含 Git 历史和未纳入索引的新文件，因此纯 `secrets` 检查不替代提交历史或本地未跟踪文件扫描。Trivy 不解析 NixOS 系统闭包；Nix 包 CVE 应在构建 `result` 后用 Vulnix 检查。两者都依赖外部漏洞数据库，其结果会随时间变化，因此不放入可复现的 `system.checks`。`nixos-rebuild build/switch` 会运行其余四项确定性源码检查。`nix flake check --no-build` 只证明输出能够求值，不能证明检查或系统闭包能够构建。

Nix 会在执行 Flake 求值和检查前把受版本控制的源码复制到通常全局可读的 Nix store。构建期 Gitleaks 可以阻止后续部署，但不能撤销这次复制；如果怀疑工作树含有凭据，应先用已可信安装的扫描器或人工检查处理，再运行 `nix develop`、`nix flake check` 或 `nixos-rebuild`。

## 配置说明

### 桌面与输入法

当前配置启用 GNOME/GDM，并提供 Niri 与 Hyprland；实际登录 session 由显示管理器和本机桌面设置决定。系统时区为 `Asia/Shanghai`，默认 locale 为 `en_US.UTF-8`，并额外支持 `zh_CN.UTF-8`。Wayland、GTK、Qt、SDL 和 GLFW 的 Fcitx 环境变量已统一配置。

### 图形与虚拟化

GPU 配置面向包含 NVIDIA GPU 的主机，启用 NVIDIA 开源内核模块（open kernel module）、硬件加速、32 位图形库和容器工具包。虚拟化模块同时启用 Podman、Incus 和 libvirt/KVM；生成器识别到的 lxcfs/Incus 运行时挂载通过 `noauto` 留给对应服务管理，避免在 `local-fs` 阶段重复挂载。`machines/nixos/default.nix` 中的用户组授予相应的宿主机管理权限，应根据实际使用者收紧。

### Nix 与更新

Flake 输入使用 `flake.lock` 锁定。更新前后都应构建完整检查集合：

```bash
nix flake check -L --keep-going
nix flake update
nix flake check -L --keep-going
sudo nixos-rebuild build --flake .#nixos
```

确认构建结果后再切换到新配置：

```bash
sudo nixos-rebuild switch --flake .#nixos
```

`system.stateVersion` 表示 NixOS 状态兼容版本，不应仅因为 nixpkgs 更新就随意修改。

## 安全与隐私注意事项

- 本仓库不应存放密码、私钥、token 或其他 secrets；部署前仍应审查自己的提交内容。
- `machines/nixos/hardware-configuration.nix` 包含主机专属文件系统 UUID。UUID 通常不是认证凭据，但会暴露主机配置特征；公开仓库使用前应确认这符合自己的隐私要求。
- systemd-boot 的启动项编辑器已关闭。若需要更强的物理访问防护，仍应另外配置 Secure Boot、磁盘加密和固件密码。
- 当前配置启用 GNOME Remote Desktop、Clash Verge TUN/service mode、Steam Remote Play 防火墙规则、Docker 兼容的 Podman socket、Incus 和 libvirt；这些是功能与权限边界，不代表系统自动完成网络隔离或安全加固。不使用时应关闭或限制。
- `machines/nixos/default.nix` 中的 `wheel`、`libvirtd` 和 `incus-admin` 用户组具有较高的宿主机管理权限，应按本机信任边界调整。
- `nixpkgs.config.allowUnfree = true` 允许使用非自由软件包；第三方软件包及 nixpkgs 仍受其各自许可证约束。

## 许可证

本仓库中的配置文件以 [MIT License](LICENSE) 发布。MIT License 仅适用于本仓库作者提供的配置内容；NixOS、nixpkgs、第三方软件包、驱动和其他资产不因本仓库采用 MIT License 而改变其原有许可证。
