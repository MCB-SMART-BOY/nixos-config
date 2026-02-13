# NixOS 配置（Flake + Home Manager）

这是一套面向日常使用与开发的 NixOS 配置，采用 Flake + Home Manager 分层结构，结构清晰、可复用、易扩展。默认桌面路线为 Niri + Wayland，同时保留 legacy 入口用于非 Flake 场景。

适合人群：
- 需要多主机、多用户管理的人
- 希望把系统层与用户层分离、模块化维护的人
- 需要代理/TUN、GPU 模式切换的人

---

## 特性概览

- Flake + Home Manager 分层
- 多主机、多用户统一管理
- Niri + Wayland 桌面体验
- 输入法与中文环境开箱可用
- 代理/TUN 与 per-user 路由方案
- GPU 特化（igpu / hybrid / dgpu）
- Waybar 支持一键切换 GPU 特化

---

## 快速开始

### 1) 一键部署（推荐）

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config
# 建议先审查代码，再执行
./run.sh
```

脚本行为要点：
- 拉取仓库并同步到 `/etc/nixos`
- 失败自动临时切换 DNS 再重试
- 默认执行 `nixos-rebuild switch --show-trace`
- 保留本机 `hardware-configuration.nix`
- 支持两种部署模式：新增/调整用户，或仅更新当前配置（保留用户/权限）
- 覆盖策略、来源策略、是否升级依赖都通过向导菜单选择
- 向导模式下除“新增用户名”外，其他配置均可通过菜单选择
- 可交互选择管理员用户（`mcb.adminUsers`）
- server profile 支持开发预设与自定义软件/虚拟化开关
- 新增未预置用户时自动生成 `home/users/<name>/default.nix` 模板
- 脚本为全交互模式：直接运行 `./run.sh` 即可

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

---

## 结构概览

```
nixos-config/
├── run.sh                    # 一键部署脚本
├── flake.nix                 # Flake 入口
├── flake.lock                # 版本锁定（可复现）
├── hosts/                    # 主机配置目录
│   ├── profiles/             # 主机配置组合
│   ├── laptop/               # 笔记本主机
│   ├── server/               # 服务器主机
│   └── nixos/                # 默认主机
├── modules/                  # 系统模块（default.nix 聚合）
├── home/                     # Home Manager 用户入口
│   ├── profiles/             # 用户配置组合
│   ├── modules/              # 用户模块拆分
│   └── users/                # 用户入口（私有配置）
├── configuration.nix         # 非 Flake 兼容入口
├── docs/                     # 项目文档
└── README.md
```

---

## 核心入口

系统层：
- 主机入口：`hosts/<hostname>/default.nix`
- 主机 Profiles：`hosts/profiles/desktop.nix` / `hosts/profiles/server.nix`
- 系统模块：`modules/*.nix`

用户层：
- 用户入口：`home/users/<user>/default.nix`
- 用户配置：`home/users/<user>/config/*`
- 用户模块：`home/modules/*.nix`

---

## GPU 模式与特化（igpu / hybrid / dgpu）

GPU 模块位于 `modules/hardware/gpu.nix`，通过 `mcb.hardware.gpu` 配置。支持 GPU 特化（specialisation），用于快速切换模式。

示例：
```nix
mcb.hardware.gpu.specialisations.enable = true;
mcb.hardware.gpu.specialisations.modes = [ "igpu" "hybrid" "dgpu" ];
```

说明：
- igpu：只用核显
- hybrid：核显 + NVIDIA（需要 busId）
- dgpu：只用独显（需硬件支持 dGPU-only 或 MUX）

重要提示：
- BIOS 若设为 dGPU-only，切换到 igpu/hybrid 可能黑屏
- 要使用 hybrid，必须补齐 iGPU/dGPU busId

### 桌面栏一键切换
Noctalia 模块 `GPU:xxx` 支持点击下拉选择，脚本路径：
- `home/users/<user>/scripts/noctalia-gpu-mode`

切换会执行 `nixos-rebuild switch --specialisation ...`，建议切换后重启系统以保证稳定。

---

## 代理 / TUN / per-user 路由

代理模式：
- `mcb.proxyMode = "tun" | "http" | "off"`

per-user 路由：
- `mcb.perUserTun.*`（按 UID 分流）

详细方案与排错请看：`docs/NETWORK_CN.md`

---

## 文档索引

- `docs/USAGE.md`：使用说明书（建议先读）
- `docs/STRUCTURE.md`：结构说明
- `docs/DETAILS.md`：细节说明（主机/模块/选项）
- `docs/NETWORK_CN.md`：国内网络问题排查

---

## 常见操作速查

- 修改主机配置：`hosts/<hostname>/default.nix`
- 修改用户名：更新主机文件与 `home/users/<user>/`
- 修改桌面快捷键：`home/users/<user>/config/niri/config.kdl`
- 修改 Waybar：`home/users/<user>/config/waybar/`
- 新增主机：在 `hosts/` 新建目录并放 `default.nix`

---

如需进一步定制（主题、输入法、脚本、包组、GPU 自动化检测等），可以在仓库内扩展模块，或直接告知需求。
