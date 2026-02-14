# 项目细节说明

该文档用于说明仓库的关键配置点与联动关系，便于维护与扩展。

## 快速定位

- 改默认用户/多用户：`hosts/<hostname>/default.nix` -> `mcb.user` / `mcb.users`
- 改系统包组：`hosts/profiles/*.nix` + `modules/packages.nix`
- 改桌面快捷键：`home/users/<user>/config/niri/config.kdl`
- 改应用主题：`home/users/<user>/config/`
- 改输入法：`modules/i18n.nix` + `home/users/<user>/config/fcitx5/profile`
- GPU 特化切换：Noctalia `GPU:xxx`（脚本：`home/users/<user>/scripts/noctalia-gpu-mode`）

---

## Home Manager 体系

- 用户入口：`home/users/<user>/default.nix`
- Profiles：`home/profiles/full.nix` / `home/profiles/minimal.nix`
- 用户专属配置：`home/users/<user>/*.nix`

常见模块：
- `home/modules/base.nix`：环境变量与 PATH
- `home/modules/programs.nix`：Alacritty/Helix
- `home/modules/desktop.nix`：niri/fuzzel/mako/waybar/swaylock
- `home/modules/shell.nix`：zsh/direnv/zoxide/starship/tmux
- `home/modules/git.nix`：git 基础配置
- `home/users/<user>/scripts.nix`：Waybar 脚本/壁纸脚本打包

配置文件存放在 `home/users/<user>/config/`，由 `xdg.configFile` 统一链接到 `~/.config`。

---

## NixOS 体系

- 系统入口：`hosts/<hostname>/default.nix`
- 核心聚合模块（不含桌面/虚拟化/游戏）：`modules/default.nix`
- 选项定义：`modules/options.nix`（`mcb.*`）
- 系统包组：`modules/packages.nix`（`mcb.packages.*`）
- 主机 Profiles：`hosts/profiles/desktop.nix` / `hosts/profiles/server.nix`

---

## 主机参数要点（mcb.*）

- `user`：默认用户
- `users`：启用 Home Manager 的用户列表
- `proxyMode`：代理模式（`tun` / `http` / `off`）
- `proxyUrl`：系统 HTTP 代理地址
- `enableProxyDns`：TUN 模式下是否强制本地 DNS
- `proxyDnsAddr` / `proxyDnsPort`
- `tunInterface` / `tunInterfaces`
- `cpuVendor`：`intel` / `amd`

---

## 桌面图形运行时（Vulkan / 非 Nix 二进制）

关键字段（`mcb.desktop.graphicsRuntime.*`）：
- `enable`：是否导出兼容运行时环境变量（`LD_LIBRARY_PATH` + `VK_DRIVER_FILES`）
- `libraryPath`：写入 `LD_LIBRARY_PATH` 的库路径列表
- `vulkanIcdDir`：Vulkan ICD 目录（会用于 `VK_DRIVER_FILES` 与 shell 兜底展开）

默认值已覆盖常见桌面场景（Steam、rustup/cargo、上游二进制包）。若你有特殊运行时需求，可在主机配置里覆盖：

```nix
mcb.desktop.graphicsRuntime = {
  enable = true;
  libraryPath = [
    "/run/current-system/sw/lib"
    "/run/opengl-driver/lib"
    "/run/opengl-driver-32/lib"
  ];
  vulkanIcdDir = "/run/opengl-driver/share/vulkan/icd.d";
};
```

---

## GPU 配置

关键字段：
- `hardware.gpu.mode`：`igpu` / `hybrid` / `dgpu`
- `hardware.gpu.igpuVendor`：`intel` / `amd`
- `hardware.gpu.prime.mode`：`offload` / `sync` / `reverseSync`
- `hardware.gpu.prime.intelBusId` / `amdgpuBusId` / `nvidiaBusId`
- `hardware.gpu.nvidia.open`
- `hardware.gpu.specialisations.*`

当前仓库策略：
- `hosts/profiles/base.nix` 默认开启 `igpu/dgpu` 特化（`mkDefault`，可被主机覆盖）
- `hosts/nixos/default.nix` 与 `hosts/laptop/default.nix` 已补齐 busId 并启用 `hybrid`
- `hosts/server/default.nix` 默认关闭 GPU 特化

GPU 示例：
```nix
# igpu only
mcb.hardware.gpu = {
  mode = "igpu";
  igpuVendor = "intel";
};

# hybrid
mcb.hardware.gpu = {
  mode = "hybrid";
  igpuVendor = "intel";
  prime = {
    mode = "offload";
    intelBusId = "PCI:0:2:0";
    nvidiaBusId = "PCI:1:0:0";
  };
};

# dgpu only
mcb.hardware.gpu = {
  mode = "dgpu";
  nvidia.open = true;
};
```

获取 busId：`lspci -D -d ::03xx`。
run.sh 选择 hybrid 时，会优先自动探测 busId（需要 `lspci`），否则回退读取主机配置；有默认值时可直接回车接受。

特化切换：
```bash
sudo nixos-rebuild switch --specialisation gpu-dgpu
```

---

## 部署脚本（run.sh）

- 负责拉取、同步、构建
- 可在向导中选择主机/用户/TUN/GPU
- 会写入 `hosts/<hostname>/local.nix` 覆盖配置
- 建议部署前先执行 `nix flake check`（现已包含脚本语法检查）

---

## 常见扩展方式

- 修改主机配置：编辑 `hosts/<hostname>/default.nix`
- 自定义应用配置：放入 `home/users/<user>/config/` 后在 `files.nix` 中接入
- 关闭系统层游戏功能：`mcb.system.enableGaming = false;`
- 音乐应用说明：`enableMusic` 会安装 `yesplaymusic`（仓库内自定义包）与 `go-musicfox`/`musicfox` 启动包装器（启动前会自动修正 `go-musicfox.ini` 的 `player.engine=mpv`、`player.mpvBin`、`unm.sources`、`unm.skipInvalidTracks`、`unm.searchLimit`）
