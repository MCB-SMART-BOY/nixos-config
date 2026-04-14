# 使用说明

## 1. 入口

```bash
nix run .#mcbctl
nix run .#mcbctl -- deploy
nix run .#mcb-deploy
```

帮助：

```bash
nix run .#mcbctl -- --help
nix run .#mcbctl -- deploy --help
nix run .#mcb-deploy -- --help
```

## 2. TUI 页面

- `Packages`
  写 `home/users/<user>/managed/packages/*.nix`
- `Home`
  写 `home/users/<user>/managed/settings/desktop.nix`
- `Users`
  写 `hosts/<host>/managed/users.nix`
- `Hosts`
  写 `hosts/<host>/managed/network.nix`、`gpu.nix`、`virtualization.nix`
- `Deploy`
  做同步和 `nixos-rebuild`
- `Actions`
  做 flake 检查、追新、同步和重建

## 3. 保存规则

`mcbctl` 现在不会再对 `managed/` 盲写。

保存时会做这些事：

1. 先跑当前页或整机级校验
2. 再确认目标文件是有效受管文件，或显式迁移后留下的受管文件
3. 如果目标文件看起来被手改破坏，直接拒绝覆盖

如果仓库来自旧树，先跑一次：

```bash
nix run .#mcbctl -- migrate-managed
nix run .#mcbctl -- extract-managed
nix run .#mcbctl -- migrate-hardware-config --host <host>
```

如果你遇到“refusing to overwrite”或“refusing to remove stale unmanaged package file”：

- 把手写内容移出 `managed/`
- 优先折叠进 `default.nix`、`packages.nix`、`local.nix`
- `extract-managed` 的自动落点是 `local.auto.nix` + `local-extracted/*.nix`
- 再重新保存

## 4. 部署

直接重建：

```bash
nix run .#mcbctl -- rebuild switch
nix run .#mcbctl -- rebuild test
nix run .#mcbctl -- rebuild boot
nix run .#mcbctl -- build-host --dry-run
```

说明：

- `rebuild switch|test|boot` 现在要求 `hosts/<host>/hardware-configuration.nix` 存在
- `build-host` 和 `rebuild build` 允许只走评估 fallback，适合 CI / 本地检查

完整向导：

```bash
nix run .#mcbctl -- deploy
```

向导负责：

- 来源选择
- 远端镜像重试
- host / user 模板生成
- `/etc/nixos` 备份与同步
- `nixos-rebuild` 执行
- GPU / 代理 / per-user TUN / 虚拟化 写回

向导行为补充：

- `返回` 现在总是回到上一个真正可交互的步骤，不会因为跳过了 TUN / GPU / server override 而卡在循环里
- 非交互模式会走保守默认：
  - `ManageUsers` 优先使用当前仓库作为本地源
  - `UpdateExisting` 默认改为远端分支 HEAD
  - 选择现有 host
  - 用户默认取 host 配置里可解析的主用户，取不到再回退环境变量和 `home/users/*`
  - 管理员默认取第一个目标用户
  - 桌面 host 自动套用识别到的 GPU 默认
  - server override 默认保持关闭
- 默认用户和 GPU Bus ID 的探测文件如果“缺失”会正常回退；如果“存在但不可读”，现在会显式告警而不是静默当成不存在
- 现有 host 的 `profile` 判定和 `per-user TUN` 默认探测现在也遵循同一规则：
  - 缺失则回退
  - 不可读或 `nix eval` 输出异常则告警后回退

## 5. 检查与追新

```bash
nix run .#mcbctl -- repo-integrity
nix run .#mcbctl -- migrate-managed
nix run .#mcbctl -- extract-managed
nix run .#mcbctl -- migrate-hardware-config --host <host>
nix run .#mcbctl -- lint-repo
nix run .#mcbctl -- doctor
nix run .#update-upstream-apps -- --check
nix run .#update-upstream-apps
```

单独刷新：

```bash
nix run .#update-zed-source
nix run .#update-yesplaymusic-source
```

## 6. 桌面命令

```bash
nix run .#lock-screen
nix run .#niri-run -- alacritty
nix run .#noctalia-gpu-mode -- --menu
nix run .#noctalia-proxy-status
```

辅助动作：

- `mcbctl terminal-action <flake-status|flake-hint|sensors|memory|disk>`
- `mcbctl screenshot-edit <full|region>`

## 7. 手写逻辑应该放哪

主机：

- `hosts/<host>/default.nix`
- `hosts/<host>/local.auto.nix` 只给 `extract-managed` 自动救援使用
- `hosts/<host>/local.nix`

用户：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/local.auto.nix` 只给 `extract-managed` 自动救援使用
- `home/users/<user>/local.nix`

模板：

- `hosts/templates/`
- `home/templates/users/`

不要把长期手写逻辑放进 `managed/`。

## 8. 验证

```bash
cargo fmt --check --manifest-path mcbctl/Cargo.toml
cargo clippy --manifest-path mcbctl/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path mcbctl/Cargo.toml
NIX_CONFIG='experimental-features = nix-command flakes' nix flake check --option eval-cache false
nix run .#mcbctl -- --help
nix run .#mcbctl -- migrate-managed --help
nix run .#mcbctl -- extract-managed --help
nix run .#mcbctl -- migrate-hardware-config --help
nix run .#mcbctl -- deploy --help
nix run .#mcb-deploy -- --help
nix run .#update-upstream-apps -- --check
```
