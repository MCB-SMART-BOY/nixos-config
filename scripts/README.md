# 脚本体系

本目录放置部署与维护脚本，建议通过仓库根目录的 `run.sh` 调用，便于统一入口和参数传递。

## 统一入口

```bash
./run.sh list
./run.sh preflight --no-network
./run.sh install --mode test
./run.sh            # 默认执行 preflight + install
./run.sh cloud      # 默认从 GitHub 拉取并部署
./run.sh sync       # 同步当前目录到云端最新版本
```

## 脚本清单

- `preflight.sh`：部署前自检（依赖、网络、硬件配置等）
- `install.sh`：本地部署（同步硬件配置、同步到 /etc/nixos、nixos-rebuild）
- `install_from_github.sh`：从 GitHub 拉取并部署
- `sync_cloud.sh`：同步当前仓库到云端最新版本
- `toolchain.sh`：安装开发工具链（rustup）
- `sync_etc.sh`：同步仓库到 `/etc/nixos`
- `sync_hardware.sh`：同步硬件配置到仓库
- `rebuild.sh`：封装 `nixos-rebuild`（支持模式/目标切换）
- `home_refresh.sh`：刷新 Home Manager systemd 服务
- `flake_update.sh`：更新 `flake.lock`
- `clean.sh`：Nix 垃圾回收（默认 dry-run）
- `status.sh`：查看仓库与系统状态
- `doctor.sh`：综合检查（preflight + 脚本语法）
- `lib.sh`：脚本公共函数库（内部使用）

> 说明：这些脚本用于目标 NixOS 机器，请在部署环境中运行。

## 默认入口的额外参数

`./run.sh` 与 `./run.sh cloud` 可以通过环境变量传递额外参数：

- `RUN_PREFLIGHT_ARGS`：传给 `preflight.sh`
- `RUN_INSTALL_ARGS`：传给 `install.sh`
- `RUN_CLOUD_ARGS`：传给 `install_from_github.sh`

示例：

```bash
RUN_PREFLIGHT_ARGS="--no-network" RUN_INSTALL_ARGS="--mode test" ./run.sh
RUN_CLOUD_ARGS="--yes --skip-preflight" ./run.sh cloud
RUN_INSTALL_ARGS="--temp-dns" ./run.sh
```
