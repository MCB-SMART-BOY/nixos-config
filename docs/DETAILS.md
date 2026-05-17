# 项目细节与联动关系

维护地图：解释"为什么这样拆、改了以后会连到哪里"。

---

## 快速定位

- 改默认用户、多用户、管理员：`local.nix` → `modules/default.nix` → `modules/users.nix`
- 改系统共享包组：`modules/options.nix`（开关定义）→ `modules/packages.nix`（包组内容）
- 改某个用户的软件：`users/<user>/packages.nix`
- 改用户界面配置：`users/<user>/config/`
- 改 Noctalia 行为：`users/<user>/noctalia.nix`、`users/<user>/scripts/`
- 改部署流程：`run.sh` + `scripts/run/`
- 改 nixpkgs 包覆盖：`overlays/default.nix`

---

## 1. 两层架构的边界

### 系统层（`modules/`）

负责"这台机器该怎么工作"：
- 系统服务（SSH、Flatpak、Docker …）
- 系统包组（CLI 工具、Wayland 运行时、调试工具 …）
- 网络、内核、缓存

所有开关定义集中在 `modules/options.nix`。各模块只读取 `config.mcb.*`，不声明自己的选项。

机器特定值通过 `local.nix`（gitignored）以 `lib.mkForce` 覆盖 `modules/default.nix` 中的 `lib.mkDefault` 默认值。

### 用户层（`users/<user>/`）

负责"这个人想怎么用机器"：
- 个人软件清单（`packages.nix`）
- dotfiles（`config/`）
- 用户脚本（`scripts/` + `scripts.nix`）
- Git 身份（`git.nix`，自包含：options + config + 身份）

---

## 2. 用户软件为什么放用户层

系统层保留共享能力（网络工具、Wayland 运行时、系统服务依赖）。
用户层负责个人软件清单（浏览器、编辑器、聊天软件、办公科研工具）。

Nix store 仍然共享构建产物，区别只在于"这个包是否出现在该用户的 profile 里"。

---

## 3. 选项声明集中管理

所有 `mcb.*` 选项在 `modules/options.nix` 中声明。这包括：

- 用户与权限：`mcb.user`、`mcb.users`、`mcb.adminUsers`、`mcb.hostRole`
- 硬件：`mcb.cpuVendor`
- Nix 构建：`mcb.nix.*`
- 代理：`mcb.proxyMode`、`mcb.proxyUrl`
- Flatpak：`mcb.flatpak.*`
- 功能开关：`mcb.gaming.enable`、`mcb.packages.*`、`mcb.virtualisation.*`
- 桌面图形：`mcb.desktop.graphicsRuntime.*`

模块文件（`gaming.nix`、`packages.nix`、`virtualization.nix` 等）只做 `config = ...`，不声明 `options`。

---

## 4. 多用户模型

关键字段：

- `mcb.user` — 主用户
- `mcb.users` — 参与 Home Manager 管理的所有用户
- `mcb.adminUsers` — 具有管理员权限（wheel）的用户

这些字段在 `local.nix` 中以 `mkForce` 设置。`modules/users.nix` 负责创建系统用户和组。

---

## 5. GPU 配置

GPU 驱动和 busId 属于硬件信息，通过 `nixos-generate-config` 写入 `hardware-configuration.nix`（gitignored）。

本项目不提供 GPU 抽象层（曾有的 `mcb.hardware.gpu.*` 已删除）。NVIDIA Prime、Intel + NVIDIA 混合等配置使用 NixOS 原生选项。

GPU wrapper 脚本（`noctalia-gpu-current`、`zed-auto-gpu`、`electron-auto-gpu`）在 `users/admin/scripts/` 中，通过 `desktop.nix` 的 `writeShellApplication` + `builtins.readFile` 打包。它们通过检测 `/run/current-system` 路径和 `/proc/cmdline` 来判断当前 GPU 模式，为应用设置正确的环境变量。

---

## 6. Noctalia 与用户脚本

- 原始脚本：`users/<user>/scripts/`
- 打包逻辑：`users/<user>/scripts.nix`（Noctalia 按钮脚本）
- GPU wrapper：`users/admin/desktop.nix`（桌面入口脚本）
- Noctalia 顶栏入口：`users/admin/noctalia.nix`

---

## 7. 部署脚本分层

- `scripts/run/cmd/` — 命令入口（deploy、add-user、switch …）
- `scripts/run/lib/vars.sh` — 共享状态变量
- `scripts/run/lib/ui.sh` — 日志、菜单、进度条
- `scripts/run/lib/env.sh` — 环境检查
- `scripts/run/lib/state.sh` — 状态重置与 PCI busId 工具
- `scripts/run/lib/targets/` — 主机、用户、覆盖项收集
- `scripts/run/lib/pipeline.sh` — 源码准备、同步、重建
- `scripts/run/lib/wizard.sh` — 交互式向导

---

## 8. 自维护包

- `pkgs/zed/` — Zed 编辑器（固定上游版本 + hash）
- `pkgs/yesplaymusic/` — YesPlayMusic（固定上游版本 + hash）

更新入口：`./pkgs/scripts/update-upstream-apps.sh`

---

## 9. 图形运行时补丁

`mcb.desktop.graphicsRuntime.*` 为从 nixpkgs 外部来的二进制（AppImage、cargo build …）提供统一的 LD_LIBRARY_PATH 和 Vulkan ICD 发现路径。

---

## 10. 维护习惯

- 改系统层之前，确认这不是用户层问题
- 加软件之前，问自己这是"共享能力"还是"个人需要"
- 动大结构之前，先跑 `nix flake check`
- 选项变更只在一处（`modules/options.nix`）
