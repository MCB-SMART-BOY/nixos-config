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

# 仅检查 Flake 和 NixOS 配置，不构建完整系统闭包
nix flake check --no-build

# 应用 nixos 主机配置
sudo nixos-rebuild switch --flake .#nixos
```

首次使用前必须检查并按目标主机修改：

- `machines/nixos/hardware-configuration.nix`：文件系统、swap 和硬件扫描结果
- `machines/nixos/hardware-gpu.nix`：GPU 驱动与硬件相关设置
- `machines/nixos/default.nix`：用户、用户组和主机专属设置
- `modules/network.nix`：主机名、代理和网络相关设置

当前主机文件定义了 `admin` 普通用户，但没有在仓库中提供密码或 SSH authorized key。部署到自己的主机时，应通过安全的本地方式设置凭据，绝不要把密码或私钥写入仓库。

`hardware-configuration.nix` 是由 `nixos-generate-config` 生成的主机专属文件，不应直接复制到另一台机器而不核对设备 UUID、文件系统和硬件模块。当前版本假定 UEFI/systemd-boot、ext4 根分区、FAT `/boot` 和主机专属 swap UUID。

## 仓库结构

```text
.
├── flake.nix                     # Flake 入口与 nixpkgs 输入
├── flake/
│   └── default.nix               # 自动发现 machines/* 并生成 nixosConfigurations
├── machines/
│   └── nixos/
│       ├── default.nix           # 主机用户与主机专属设置
│       ├── system.nix             # 主机系统架构
│       ├── hardware-configuration.nix
│       └── hardware-gpu.nix
└── modules/
    ├── default.nix               # 模块总入口
    ├── boot.nix                  # systemd-boot、内核
    ├── desktop.nix               # 桌面、显示管理器、Portal
    ├── i18n.nix                  # locale、时区、fcitx5
    ├── packages.nix              # 系统软件包
    ├── virtualisation.nix        # Podman、Incus、libvirt
    ├── game.nix                  # Steam、GameMode
    ├── applications.nix          # PipeWire、蓝牙、Flatpak 等
    ├── network.nix               # NetworkManager、Clash Verge
    ├── security.nix              # AppArmor、auditd、polkit、sudo
    ├── nix.nix                   # Nix 设置、GC、zram swap
    ├── lib.nix                   # 公共 Nix 辅助函数
    └── core.nix                  # nix-ld 运行库
```

`flake/default.nix` 会扫描 `machines/` 下同时包含目录和 `default.nix` 的条目，并为每个条目生成一个 NixOS 配置。新增主机时，应保持该目录结构，并提供对应的 `default.nix`；`system.nix` 可用于指定目标架构。

## 配置说明

### 桌面与输入法

当前配置启用 GNOME/GDM，并提供 Niri 与 Hyprland；实际登录 session 由显示管理器和本机桌面设置决定。系统时区为 `Asia/Shanghai`，默认 locale 为 `en_US.UTF-8`，并额外支持 `zh_CN.UTF-8`。Wayland、GTK、Qt、SDL 和 GLFW 的 Fcitx 环境变量已统一配置。

### 图形与虚拟化

GPU 配置面向包含 NVIDIA GPU 的主机，启用 NVIDIA 开源内核模块（open kernel module）、硬件加速、32 位图形库和容器工具包。虚拟化模块同时启用 Podman、Incus 和 libvirt/KVM；`machines/nixos/default.nix` 中的用户组授予相应的宿主机管理权限，应根据实际使用者收紧。

### Nix 与更新

Flake 输入使用 `flake.lock` 锁定。更新依赖前建议先检查当前配置，并在更新后重新执行检查：

```bash
nix flake check --no-build
nix flake update
nix flake check --no-build
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
