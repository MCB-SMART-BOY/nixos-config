# scripts-rs

这个目录不是“顺手把 Shell 翻译成 Rust”那么简单。

它现在已经是仓库正式在用的脚本实现层：

- 部署向导走 `run-rs`
- release 流程走 `run-rs release`
- 用户命令由这里编译出的二进制提供
- 官网应用追新也走这里的工具

换句话说，`scripts-rs` 现在不是未来计划，而是当前事实。

## 这个目录在整个仓库里的位置

你可以这样理解：

- `scripts-rs/src/bin/*.rs`
  具体命令实现
- `scripts-rs/src/lib.rs`
  公共复用逻辑
- `pkgs/scripts-rs/default.nix`
  把这些 Rust 二进制打成 Nix 包
- `home/users/<user>/scripts.nix`
  把其中一部分命令链接到用户环境里

所以如果你改的是命令行为，重点看这里。
如果你改的是“这个命令怎么进入系统或用户环境”，就要连同 `pkgs/scripts-rs/` 和 `home/users/<user>/scripts.nix` 一起看。

## 当前比较关键的命令

- `run-rs`
  完整部署 / release 流程
- `noctalia-gpu-mode-rs`
  GPU specialisation 菜单与切换
- `lock-screen-rs`
  锁屏入口
- `niri-run-rs`
  Niri 会话辅助启动
- `noctalia-*-rs`
  一组给状态栏和桌面交互用的命令
- `update-zed-source-rs`
  更新 Zed 官网稳定版 pin
- `update-yesplaymusic-source-rs`
  更新 YesPlayMusic 官网稳定版 pin
- `update-upstream-apps-rs`
  一次更新上述两个 pin

## 在仓库里怎么用

如果你在仓库根目录，最推荐的方式是：

```bash
nix run .#run-rs
```

只想检查追新：

```bash
nix run .#update-upstream-apps -- --check
```

想实际更新：

```bash
nix run .#update-upstream-apps
```

这条路径的好处是，你走的就是仓库现在真正暴露给外部的入口，而不是只在本地 `cargo run` 一次。

## 本地开发怎么做

### 只做编译检查

```bash
cd scripts-rs
cargo check
```

### 运行某个命令

```bash
cd scripts-rs
cargo run --bin run-rs
```

或者：

```bash
cd scripts-rs
cargo run --bin noctalia-gpu-mode-rs
```

### 格式化

```bash
cd scripts-rs
cargo fmt
```

## 什么时候该改 `src/lib.rs`

如果你发现多个命令都在重复做这些事情，就应该考虑提到 `src/lib.rs`：

- 查找仓库根目录
- 调外部命令
- 输出状态栏 JSON
- 解析 GPU 模式
- 处理 XDG 路径

这一步做得越早，后面维护就越轻。

## 这条路线真正带来的好处

把脚本迁到 Rust，价值不在“换一种语言”，而在这些更实际的东西：

- 复杂流程不再依赖 Bash 的隐式行为
- 可以做编译期检查
- 复用逻辑更容易抽出来
- 更适合长期维护，而不是把一次性脚本越堆越大

如果你后面要继续扩展部署流程、Noctalia 命令或者追新工具，优先继续沿着这条路线写就对了。
