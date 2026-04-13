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
2. 再确认目标文件是受管文件、旧占位文件，或可安全迁移的旧格式
3. 如果目标文件看起来被手改破坏，直接拒绝覆盖

如果你遇到“refusing to overwrite”或“refusing to remove stale unmanaged package file”：

- 把手写内容移出 `managed/`
- 改到 `default.nix`、`packages.nix`、`local.nix`
- 再重新保存

## 4. 部署

直接重建：

```bash
nix run .#mcbctl -- rebuild switch
nix run .#mcbctl -- rebuild test
nix run .#mcbctl -- rebuild boot
nix run .#mcbctl -- build-host --dry-run
```

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

## 5. 检查与追新

```bash
nix run .#mcbctl -- repo-integrity
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
- `hosts/<host>/local.nix`

用户：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
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
nix run .#mcbctl -- deploy --help
nix run .#mcb-deploy -- --help
nix run .#update-upstream-apps -- --check
```
