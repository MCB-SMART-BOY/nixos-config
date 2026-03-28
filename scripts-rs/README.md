# scripts-rs

本目录提供项目现有 Shell 脚本的 Rust 对应实现（额外版本，不替换原脚本）。

## 构建

```bash
cd scripts-rs
cargo build --release
```

## 对应关系

- `run.sh` -> `run-rs`（Rust 入口，委托执行 `run.sh`）
- `home/users/mcbnixos/scripts/lock-screen` -> `lock-screen-rs`
- `home/users/mcbnixos/scripts/niri-run` -> `niri-run-rs`
- `home/users/mcbnixos/scripts/noctalia-bluetooth` -> `noctalia-bluetooth-rs`
- `home/users/mcbnixos/scripts/noctalia-cpu` -> `noctalia-cpu-rs`
- `home/users/mcbnixos/scripts/noctalia-disk` -> `noctalia-disk-rs`
- `home/users/mcbnixos/scripts/noctalia-flake-updates` -> `noctalia-flake-updates-rs`
- `home/users/mcbnixos/scripts/noctalia-gpu-mode` -> `noctalia-gpu-mode-rs`（Rust 入口，委托执行原脚本）
- `home/users/mcbnixos/scripts/noctalia-memory` -> `noctalia-memory-rs`
- `home/users/mcbnixos/scripts/noctalia-net-speed` -> `noctalia-net-speed-rs`
- `home/users/mcbnixos/scripts/noctalia-net-status` -> `noctalia-net-status-rs`
- `home/users/mcbnixos/scripts/noctalia-power` -> `noctalia-power-rs`
- `home/users/mcbnixos/scripts/noctalia-proxy-status` -> `noctalia-proxy-status-rs`
- `home/users/mcbnixos/scripts/noctalia-temperature` -> `noctalia-temperature-rs`
- `home/users/mcbnixos/scripts/wallpaper-random` -> `wallpaper-random-rs`
- `pkgs/zed/scripts/update-source.sh` -> `update-zed-source-rs`
- `pkgs/yesplaymusic/scripts/update-source.sh` -> `update-yesplaymusic-source-rs`
- `pkgs/scripts/update-upstream-apps.sh` -> `update-upstream-apps-rs`

## 说明

- `*-rs` 命令默认从当前目录向上查找仓库根（`flake.nix` + `pkgs/`）。
- `run-rs` 找不到 `run.sh` 时可通过 `RUN_SH_PATH` 显式指定。
- `noctalia-gpu-mode-rs` 找不到原脚本时可通过 `NOCTALIA_GPU_MODE_SH` 显式指定。
