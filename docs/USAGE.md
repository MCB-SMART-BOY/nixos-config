# 使用说明

按真实场景写的维护手册：刚装完 NixOS 怎么落配置、日常怎么改、怎么加用户。

---

## 1. 第一次部署

先确认三件事：

- `hardware-configuration.nix` 已生成（`sudo nixos-generate-config`）
- 仓库已 clone 到本地
- `local.nix` 已填写（可复制 `local.nix.example` 开始）

推荐流程：

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config
cp local.nix.example local.nix   # 编辑：填你的用户名、主机名、时区
sudo nixos-generate-config       # 生成 /etc/nixos/hardware-configuration.nix
sudo nixos-rebuild switch --flake .#host
```

`./run.sh` 提供交互式向导作为替代。

---

## 2. 日常维护

### 2.1 改配置后切换

```bash
sudo nixos-rebuild switch --flake .#host
```

### 2.2 先试跑不落地

```bash
sudo nixos-rebuild test --flake .#host
```

### 2.3 更新 flake 输入

```bash
nix flake update
sudo nixos-rebuild switch --flake .#host
```

### 2.4 健康检查

```bash
nix flake check
```

---

## 3. 给某个用户加软件

去改这个用户的 `packages.nix`：

```
users/<user>/packages.nix
```

原则：
- **系统共享的东西放系统层**（`modules/packages.nix`）
- **只属于某个用户的东西放用户层**（`users/<user>/packages.nix`）
- 构建产物由 Nix store 共享，不会重复安装

---

## 4. 新增用户

```bash
./run.sh add-user bob --admin           # 管理员
./run.sh add-user alice                  # 普通用户
./run.sh add-user eve --copy-from admin  # 从 admin 复制配置模板
```

生成的 `users/<user>/` 目录是自包含的，新用户有独立的软件清单和 dotfiles。

---

## 5. 系统层和用户层，别混

### 系统层（`modules/`）回答「这台机器应该是什么样」

- 主机名、用户列表、管理员
- 系统服务（SSH、Flatpak、Docker …）
- 系统共享包组
- 网络、缓存、内核参数

所有开关定义集中在 `modules/options.nix`。

### 用户层（`users/<user>/`）回答「这个人想怎么用」

- 个人软件清单
- Niri / Noctalia / 终端 / 编辑器配置
- 主题、快捷键

---

## 6. GPU 配置

GPU 驱动和 busId 属于硬件信息，应写入 `hardware-configuration.nix`（由 `nixos-generate-config` 生成）。

NVIDIA Prime / Intel + NVIDIA 混合等配置使用 NixOS 原生选项（`hardware.nvidia.prime` 等），本项目不再提供 GPU 抽象层。

如果需要 iGPU / dGPU / hybrid 切换，使用 NixOS 原生 `specialisation` 语法在 `hardware-configuration.nix` 中声明。

---

## 7. 代理

三种模式（在 `local.nix` 中设置）：

```nix
mcb.proxyMode = "off";   # 不使用代理（默认）
mcb.proxyMode = "tun";   # TUN 模式（由 clash-verge-rev GUI 自行管理）
mcb.proxyMode = "http";  # HTTP 代理
```

TUN 模式下的路由和 DNS 由 clash-verge-rev 的 GUI 一键管理，Nix 层只保留 NetworkManager + 防火墙，不干预策略路由。

中国大陆用户建议设置缓存镜像：

```nix
mcb.nix.cacheProfile = "cn";
```

---

## 8. 回滚

```bash
sudo nixos-rebuild switch --rollback
```

或重启后在 systemd-boot 菜单选旧 generation。

---

## 9. 改什么去哪

| 想改的 | 去 |
|---|---|
| 主机名、用户名、时区 | `local.nix` |
| GPU / 磁盘 / 硬件 | `hardware-configuration.nix` |
| 系统包组 | `modules/packages.nix` |
| 防火墙 / DNS | `modules/networking.nix` |
| 内核 / sysctl | `modules/boot.nix` |
| 某个用户的软件 | `users/<user>/packages.nix` |
| dotfiles | `users/<user>/config/` |
| 部署流程 | `run.sh` + `scripts/run/` |
| 选项开关定义 | `modules/options.nix` |
