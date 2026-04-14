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

- `Overview`
  汇总当前 host、repo / doctor 健康、dirty 状态和推荐主动作
- `Apply`
  做当前 host 的默认应用预览、同步和 `nixos-rebuild`
- `Inspect`
  看健康详情，并执行 `flake check` / 上游 pin 检查等 Inspect 动作
- `Packages`
  写 `home/users/<user>/managed/packages/*.nix`
- `Home`
  写 `home/users/<user>/managed/settings/desktop.nix`
- `Users`
  写 `hosts/<host>/managed/users.nix`
- `Hosts`
  写 `hosts/<host>/managed/network.nix`、`gpu.nix`、`virtualization.nix`
- `Actions`
  迁移期入口页，把历史动作按 `Inspect / Apply / Advanced` 分组跳转

当前页语义补充：

- `Overview`：`Enter` 打开推荐主动作，`a` 尝试直接执行当前 Apply，`p` 打开 Apply，`i` 打开 Inspect，`r/d/R` 刷新健康项
- `Overview`：健康区现在也会汇总 `Packages / Home / Users / Hosts` 的 `受管保护` 快照，进入具体页面前就能看到哪个目标会阻塞保存
- `Overview`：当推荐主动作变成 `Review Save Guards` 时，`Enter` 会直接跳到最该先处理的页面；host runtime 分片问题优先跳 `Hosts`，host users/default 问题优先跳 `Users`
- `Overview`：跳过去后还会把左侧焦点尽量落到最相关的字段或组过滤，而不是只切页
- `Apply`：左侧先看执行门槛和命令预览，右侧再调整高级字段；打开高级模式后，右下角会出现 `Advanced Workspace`，可用 `J/K` 选高级动作、`X` 执行；`x` 仍按当前 Apply 路径处理
- `Inspect`：`j/k` 选检查动作，`r/d/R` 刷新健康项，`x` 执行当前 Inspect 动作；右上角 `Health Details` 现在会带上四条写回链的 `受管保护` 快照和阻塞细节
- `Packages / Home / Users / Hosts`：右侧摘要现在会提前显示 `受管保护` 状态；如果同一 `managed` 子树里已有损坏分片，或者 `managed/packages/` 里混入了非受管陈旧组文件，会先在页面里提示
- `Actions`：`Enter/Space/x` 现在都只打开归宿页；`Advanced` 动作会跳到 Apply 页里的 `Advanced Workspace`

## 3. 保存规则

`mcbctl` 现在不会再对 `managed/` 盲写。

保存时会做这些事：

1. 先跑当前页或整机级校验
2. 再确认目标文件是有效受管文件，或显式迁移后留下的受管文件
3. `Home` / `Users` / `Hosts` 还会顺带检查同一 `managed` 子树里的兄弟分片是否仍然有效
4. 如果目标文件或兄弟分片看起来被手改破坏，直接拒绝覆盖

保存失败时，TUI 现在会保留 dirty 状态并把原因写到状态栏，不会直接退出。

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
- 现有 `home/users/*` 枚举如果目录结构异常，也会显式告警后回退；GPU 自动识别优先尝试 `lspci -D`，命令缺失时静默降级，命令执行异常时告警后回退

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

## 7. 发布

```bash
nix run .#mcb-deploy -- release
```

发布时如果版本探测、上一个 tag 探测或 git log 生成失败，现在会显式告警，并退回到保守版本号或保守 release notes，而不是静默生成空结果。

## 8. 手写逻辑应该放哪

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

## 9. 验证

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
