# scripts-rs

这个目录的存在，不是为了“把 Shell 换个语言重写一遍”这么简单。

它更像是给这套仓库补一条新的脚本路线：

- 保留原本能工作的 Shell 方案
- 同时把核心脚本逐步迁到 Rust
- 让脚本逻辑更容易测试、编译检查和长期维护

如果你本来就更偏爱 Rust，不喜欢把越来越复杂的流程继续堆在 Bash 里，这个目录就是为你准备的。

---

## 当前定位

这里的 `*-rs` 命令，是项目现有脚本的 Rust 对应实现。

需要特别说明两点：

- `run-rs` 现在已经具备完整部署 / release 流程实现
- `noctalia-gpu-mode-rs` 现在也已经是纯 Rust 实现

但仓库当前默认文档和部署入口，仍然优先写：

```bash
./run.sh
```

原因不是 Rust 版不能用，而是主仓库的默认工作流仍然以 `run.sh` 为中心。
换句话说，Rust 路线已经能走，只是还没有把 Shell 路线从仓库主入口里彻底撤掉。

---

## 构建

```bash
cd scripts-rs
cargo build --release
```

只想做本地检查：

```bash
cd scripts-rs
cargo check
```

仓库级的 `nix flake check` 也会顺手跑这里的 `cargo check`。

---

## 对应关系

- `run.sh` -> `run-rs`
- `home/users/mcbnixos/scripts/lock-screen` -> `lock-screen-rs`
- `home/users/mcbnixos/scripts/niri-run` -> `niri-run-rs`
- `home/users/mcbnixos/scripts/noctalia-bluetooth` -> `noctalia-bluetooth-rs`
- `home/users/mcbnixos/scripts/noctalia-cpu` -> `noctalia-cpu-rs`
- `home/users/mcbnixos/scripts/noctalia-disk` -> `noctalia-disk-rs`
- `home/users/mcbnixos/scripts/noctalia-flake-updates` -> `noctalia-flake-updates-rs`
- `home/users/mcbnixos/scripts/noctalia-gpu-mode` -> `noctalia-gpu-mode-rs`
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

---

## 怎么理解这套目录

### `src/bin/*.rs`

每个文件对应一个可执行命令。
如果你想改某个具体脚本行为，通常就是进这里找对应名字。

### `src/lib.rs`

放公共函数和复用逻辑。
如果你发现多个二进制都在复制同一套命令执行、路径探测、JSON 输出逻辑，那它就应该被提到这里。

---

## 推荐使用方式

### 我只想部署系统

继续优先用：

```bash
./run.sh
```

这是当前仓库对外最稳定、最直接的入口。

### 我在维护脚本体系，想优先走 Rust

你可以直接运行：

```bash
cd scripts-rs
cargo run --bin run-rs
```

或者：

```bash
cd scripts-rs
cargo run --bin noctalia-gpu-mode-rs
```

### 我在改用户脚本

要注意一个现实情况：

- `home/users/<user>/scripts.nix` 当前默认仍然打包 Shell 版用户脚本
- `scripts-rs/` 里的 Rust 版更适合独立运行、调试和后续替换

所以如果你改的是“仓库默认用户脚本打包链”，就不能只改这里而不看 `scripts.nix`。

---

## 这条路线真正的价值

把脚本迁到 Rust，不是为了写得更长，而是为了几件很实际的事：

- 让复杂逻辑不再依赖 Bash 的隐式行为
- 能做编译期检查
- 更容易拆复用逻辑
- 更容易把脚本做成长期可维护的工具，而不是一次性流程

如果这套仓库后面继续往“Rust 成为主脚本路线”走，这个目录就是基础。
