# 使用说明书

本手册面向初次使用或需要快速维护的用户，覆盖：从零部署、主机与用户设置、GPU 特化、代理/TUN、日常更新与故障排除。

---

## 0. 部署前检查

- 确保 `hardware-configuration.nix` 已生成
- 确保 `nixos-rebuild` 可用
- 网络不稳定时先参考 `docs/NETWORK_CN.md`

---

## 1. 从零部署详细步骤

以下步骤适用于全新安装或从空目录开始部署。

### 1.1 安装 NixOS 基础系统

- 使用官方 ISO 安装 NixOS（图形或最小化均可）
- 按向导完成分区、挂载与基础安装
- 完成后进入新系统

### 1.2 生成硬件配置

```bash
sudo nixos-generate-config
```

确认硬件配置存在：
- `/etc/nixos/hardware-configuration.nix`

你也可以将硬件配置放到：
- `/etc/nixos/hosts/<hostname>/hardware-configuration.nix`

### 1.3 获取配置仓库

方式 A：使用一键部署脚本（推荐）
```bash
curl -fsSL -o run.sh https://raw.githubusercontent.com/MCB-SMART-BOY/nixos-config/master/run.sh
chmod +x run.sh
./run.sh
```

方式 B：手动克隆到 /etc/nixos
```bash
sudo mv /etc/nixos /etc/nixos.backup.$(date +%Y%m%d-%H%M%S) 2>/dev/null || true
sudo git clone https://github.com/MCB-SMART-BOY/nixos-config.git /etc/nixos
```

### 1.4 创建/选择主机目录

如果是新主机，创建：
```bash
sudo mkdir -p /etc/nixos/hosts/<hostname>
```

最小主机入口示例：
```nix
# /etc/nixos/hosts/<hostname>/default.nix
{ config, lib, pkgs, ... }:
{
  imports = [ ../profiles/desktop.nix ./hardware-configuration.nix ];

  mcb = {
    user = "youruser";
    users = [ "youruser" ];
    proxyMode = "off";
  };

  networking.hostName = "<hostname>";
  system.stateVersion = "25.11";
}
```

### 1.5 应用配置

```bash
cd /etc/nixos
sudo nixos-rebuild switch --flake .#<hostname>
```

完成后重启系统。

---

## 2. 主机与用户入口

主机入口：`hosts/<hostname>/default.nix`
用户入口：`home/users/<user>/default.nix`

常用字段：
```nix
mcb.user = "mcbnixos";
mcb.users = [ "mcbnixos" "mcblaptopnixos" ];
```

说明：
- 单用户可只设置 `mcb.user`
- 多用户建议写 `mcb.users`

---

## 3. 一键部署

```bash
./run.sh
```

脚本会引导你选择：
- 主机（hosts 目录）
- 用户列表
- 是否启用 per-user TUN
- GPU 模式（可选）

默认行为：
- 交互模式未指定覆盖策略时，默认 `--ask`
- 非交互模式未指定覆盖策略时，默认 `--backup`
- 默认不附加 `--upgrade`，需要时可显式传入 `./run.sh --upgrade`

脚本会写入 `hosts/<hostname>/local.nix` 做临时覆盖，不会破坏你的主配置。

---

## 4. 日常更新与回滚

更新：
```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

测试构建：
```bash
sudo nixos-rebuild test --flake .#<hostname>
```

回滚（系统级）：
- 重启后在 systemd-boot 中选择旧的 generation
- 或使用：
  ```bash
  sudo nixos-rebuild switch --rollback
  ```

---

## 5. GPU 模式与特化

### 5.1 启用特化
默认在 `hosts/profiles/base.nix` 启用了 `igpu/dgpu` 特化（可被主机覆盖）。

启用 hybrid 需要补 busId：
```nix
mcb.hardware.gpu = {
  igpuVendor = "intel";
  prime = {
    intelBusId = "PCI:0:2:0";
    nvidiaBusId = "PCI:1:0:0";
  };
  specialisations.modes = [ "igpu" "hybrid" "dgpu" ];
};
```

说明：
- busId 格式为 `PCI:<bus>:<device>:<function>`（可由 `lspci -D -d ::03xx` 获得）
- 使用 `run.sh` 向导选择 hybrid 时，会优先自动探测 busId（需要 `lspci`），否则回退读取主机配置；有默认值时可直接回车接受

### 5.2 切换方式
开机选择：systemd-boot 中选择 `gpu-igpu` / `gpu-hybrid` / `gpu-dgpu`

命令切换：
```bash
sudo nixos-rebuild switch --specialisation gpu-igpu
sudo nixos-rebuild switch --specialisation gpu-hybrid
sudo nixos-rebuild switch --specialisation gpu-dgpu
```

桌面栏一键切换：
- 模块 `GPU:xxx` 可点开下拉选择
- 脚本路径：`home/users/<user>/scripts/noctalia-gpu-mode`

注意：
- BIOS 若设为 dGPU-only，切换到 igpu/hybrid 可能黑屏
- 切换后建议重启

---

## 6. 代理 / TUN / per-user 路由

关键选项：
```nix
mcb.proxyMode = "tun";      # tun/http/off
mcb.tunInterface = "Meta";
mcb.tunInterfaces = [ "Meta" "Mihomo" "clash0" ];
```

per-user TUN 示例：
```nix
mcb.proxyMode = "tun";
mcb.enableProxyDns = false;
mcb.users = [ "mcbnixos" "mcblaptopnixos" ];
mcb.perUserTun.enable = true;
mcb.perUserTun.redirectDns = true;
mcb.perUserTun.interfaces = {
  mcbnixos = "Meta";
  mcblaptopnixos = "Mihomo";
};
mcb.perUserTun.dnsPorts = {
  mcbnixos = 1053;
  mcblaptopnixos = 1054;
};
```

更多排查见 `docs/NETWORK_CN.md`。

---

## 7. 维护流程与最佳实践

推荐流程（每次更新时）：
1. 拉取或编辑配置，保持变更小而清晰
2. 如需更新依赖：`nix flake update`
3. 使用 `nixos-rebuild test` 先测试
4. 通过后再 `nixos-rebuild switch`
5. 核心组件更新后重启系统

最佳实践：
- 变更前先 `git status`，保持提交粒度小
- 大改动前备份 `/etc/nixos` 或创建 Git 标签
- 使用 `hosts/<hostname>/local.nix` 放主机私有覆盖
- `hardware-configuration.nix` 不随意迁移到其他主机
- per-user TUN 的接口名与 DNS 端口要唯一且对应
- hybrid 模式必须有准确 busId
- 需要验证时用 `nixos-rebuild build --show-trace` 先排错

清理建议：
- 系统已配置定期 GC，手动清理可用：
  ```bash
  sudo nix-collect-garbage -d
  ```

---

## 8. 常见操作速查

- 修改主机配置：`hosts/<hostname>/default.nix`
- 修改桌面快捷键：`home/users/<user>/config/niri/config.kdl`
- 修改 Waybar：`home/users/<user>/config/waybar/`
- 修改包组开关：`hosts/profiles/*.nix` + `modules/packages.nix`
- 新增主机：在 `hosts/` 新建目录并放 `default.nix`

---

## 9. 故障排除

- 构建失败：先 `sudo nixos-rebuild build --show-trace` 看报错
- DNS 解析失败：检查 `mcb.proxyMode` 与 `mcb.proxyDnsPort` 设置
- GPU 黑屏：检查 BIOS 是否 dGPU-only；回退到 `gpu-igpu` 或改回 Hybrid/Auto
- 报错 `Module ... has an unsupported attribute 'assertions'`：模块若使用 `config = ...` 结构，`assertions` 需放在 `config` 内（或移除 `config =` 直接使用顶层）

---

如需进一步定制（主题、输入法、脚本、包组、GPU 自动化检测等），可以直接扩展模块。
