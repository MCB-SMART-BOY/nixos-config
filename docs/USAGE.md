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

## 2. TUI 顶层区域

- `Overview`
  汇总当前 host、repo / doctor 健康、dirty 状态和推荐主动作
- `Edit`
  承接 `Packages / Home / Users / Hosts` 四个受管编辑页
- `Apply`
  做当前 host 的默认应用预览、同步和 `nixos-rebuild`
- `Advanced`
  承接高级部署与维护入口；现在已经是独立顶层区域，进入后会显示推荐高级动作、进入原因和完成后的返回路径；左侧预览和中间面板都会按当前高级动作自适应，仓库维护看 `Maintenance Preview + Repository Context`，完整向导才看 `Deploy Preview + Deploy Parameters`，其中 `RemotePinned` 会额外显示 `固定 ref` 输入；`x/X` 默认执行当前高级动作，`b` 返回 `Apply`
- `Inspect`
  看健康详情，并执行 `flake check` / 上游 pin 检查等 Inspect 动作

`Edit` 区内的具体写回页仍然是：

- `Packages`
  写 `home/users/<user>/managed/packages/*.nix`
- `Home`
  写 `home/users/<user>/managed/settings/desktop.nix`
- `Users`
  写 `hosts/<host>/managed/users.nix`
- `Hosts`
  写 `hosts/<host>/managed/network.nix`、`gpu.nix`、`virtualization.nix`

当前页语义补充：

- `Tab / Shift-Tab`：现在只在 `Overview / Edit / Apply / Advanced / Inspect` 五个顶层区域间切换
- `Edit`：顶部现在有可见的 `Packages / Home / Users / Hosts` 子导航；`1/2/3/4` 依次切页，子导航下面会汇总当前目标和四页 dirty
- `Edit`：区域摘要现在会给出唯一推荐下一步，优先指出“当前页先保存”还是“先切到哪一页处理 dirty / 受管保护”
- `Overview`：`Enter` 打开推荐主动作，`a` 尝试直接执行当前 Apply，`p` 打开 Apply，`i` 打开 Inspect，`r/d/R` 刷新健康项
- `Overview`：健康区现在也会汇总 `Packages / Home / Users / Hosts` 的 `受管保护` 快照，进入具体页面前就能看到哪个目标会阻塞保存
- `Overview`：当推荐主动作变成 `Review Save Guards` 时，`Enter` 会直接跳到最该先处理的页面；host runtime 分片问题优先跳 `Hosts`，host users/default 问题优先跳 `Users`
- `Overview`：跳过去后还会把左侧焦点尽量落到最相关的字段或组过滤，而不是只切页
- `Apply`：左侧先看执行门槛和命令预览，右侧再调整 `Apply Controls`；`x` 按当前默认 Apply 路径处理
- `Advanced`：左侧先看区域摘要和动作预览，再决定当前高级动作；现在它已经走独立的 `Page::Advanced` 叶子和独立按键分支，不再与 `Apply` 共用同一个叶子页，也不再依赖 `show_advanced` 这个 Apply 兼容布尔开关来表示“当前在 Advanced”；摘要会说明“当前任务 / 推荐动作 / 为什么在这里做 / 做完后回哪”；仓库维护动作会显示 `Maintenance Summary + Maintenance Preview + Repository Context + Maintenance Detail`，并且不再复用 deploy 参数或 Apply 告警；完整向导动作会显示 `Advanced Summary + Deploy Preview + Deploy Parameters + Deploy Wizard Detail`，而且这些参数现在不只焦点独立，`host / mode / action / source / ref / upgrade` 这组值也会独立保存；当来源是 `RemotePinned` 时，`Deploy Parameters` 会额外要求填写 `固定 ref`，并在进入 `mcb-deploy` 时用内部参数显式带过去；动作列表会按推荐分组优先级自动排序；`J/K` 选高级动作，`x/X` 执行，`b` 返回 `Apply`
- `Inspect`：`j/k` 选检查动作，`r/d/R` 刷新健康项，`x` 执行当前 Inspect 动作；右上角 `Health Details` 现在会带上四条写回链的 `受管保护` 快照和阻塞细节
- `Packages / Home / Users / Hosts`：右侧摘要现在会提前显示 `受管保护` 状态；如果同一 `managed` 子树里已有损坏分片，或者 `managed/packages/` 里混入了非受管陈旧组文件，会先在页面里提示
- `Packages`：除了分类 / 来源 / 组过滤，现在还支持按项目工作流过滤；`workflow` 用来表达这个仓库为什么推荐某个软件，而不是重复做一份 nixpkgs 搜索目录；切换工作流后，左侧摘要会直接显示当前工作流的说明、可选数量和已选数量，右侧 `Package Context` 也会把当前 workflow 下“已选 / 未选”的差异直接列出来；`A` 会先预览当前 workflow 下尚未选中的软件，`Enter` 确认后才批量加入当前用户选择；真正写回仍然要按 `s`；批量加入成功后，右侧会直接显示最近动作、下一步和最近结果
- 历史 `Actions` 已降为迁移期内部模块，不再作为顶层区域继续暴露

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
- server host 如果从 summary 返回到 server override，再改成“沿用主机现有配置”，旧 override 字段会被清空，不会带着上一轮的值继续执行
- 同步前和重建前各有一次确认；任一处输入 `n` 都会退出当前部署，但仍会执行临时 DNS / 临时目录收尾
- 如果主流程已经完成而收尾清理失败，部署会显式以 cleanup 错误结束，不会把这次执行当成成功
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
- 本地源和远端源的 `git rev-parse HEAD` 都只用于补充显示 `source_commit`；探测失败会显式告警并继续后续复制/拉取，不会阻塞部署，也不会保留上一轮旧 commit
- `/etc/nixos` 备份时间戳优先尝试 `date +%Y%m%d-%H%M%S`；探测失败会显式告警并回退到 `unknown`
- `mcbctl/Cargo.toml` 存在时，部署前会额外尝试一次 `cargo check --quiet` 作为 Rust 编译自检；这是一条增强型检查，`cargo check` 失败会中止流程，但缺少 `cargo` 或仓库里没有 `mcbctl/Cargo.toml` 只会告警并跳过
- rootless 模式判断目录是否可写时，会临时写入 `.mcbctl-write-*` 探针文件；该探针删除是 best-effort，不属于部署收尾清理，删除失败不会升级成业务错误
- `MCBCTL_COPY_USER_TEMPLATE` 是可选模板复制开关；只有字面值 `true` 会复制模板用户目录里的 `config/assets/files.nix`，缺失或其他值都等价于关闭
- 如果 GPU 拓扑无法自动识别，TTY 流程会退回手动模式，但仍然允许直接手填 iGPU / NVIDIA Bus ID，不会因为候选列表为空而卡死

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

发布默认会使用当前 `mcbctl` 包版本作为 release tag；如果要发新版本，先更新 `mcbctl/Cargo.toml` 和 `pkgs/mcbctl/default.nix`，或者显式传入 `RELEASE_VERSION`。

发布时如果上一个 tag 探测或 git log 生成失败，现在会显式告警，并退回到保守 release notes，而不是静默生成空结果。
发布前还会强制探测 `git status --porcelain`；如果探测失败会直接中止发布，避免把未知工作区状态误判成干净。
如果需要给外部平台消费当前 release 资产，也可以单独运行 `nix run .#mcbctl -- release-manifest [--version <tag>]` 输出机器可读的 release 资产清单。

创建 GitHub Release 后，CI 资产工作流会按刚创建的 tag 触发，而不是按当前分支 head 触发；这样 release 页面、`release-manifest.json` 和上传的预编译资产始终对齐。

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
