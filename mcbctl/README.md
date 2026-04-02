# mcbctl

这个目录不是“顺手把 Shell 翻译成 Rust”那么简单。

它现在已经是仓库正式在用的脚本实现层：

- TUI 控制台走 `mcbctl`
- 直接部署 / release 走 `mcb-deploy`
- 用户命令由这里编译出的二进制提供
- 官网应用追新也走这里的工具

换句话说，`mcbctl` 现在不是未来计划，而是当前事实。

## 这个目录在整个仓库里的位置

你可以这样理解：

- `mcbctl/src/bin/*.rs`
  具体命令实现；现在已经按领域分到 `control/`、`network/`、`desktop/`、`music/`、`noctalia/`、`update/`
- `mcbctl/src/lib.rs`
  公共复用逻辑
- `mcbctl/src/domain/`
  共享的数据模型、枚举和领域对象；`DeployPlan` 这类跨 TUI / CLI 的部署计划对象也在这里
- `mcbctl/src/store/`
  `catalog/`、`managed/`、主机探测等 I/O 逻辑；`store/deploy.rs` 现在也负责共享的仓库同步计划、`nixos-rebuild` 计划与执行
- `mcbctl/src/tui/views/`
  TUI 渲染层；页面渲染已拆成独立文件
- `mcbctl/src/tui/state.rs`
  当前的顶层状态机和跨页业务编排
- `mcbctl/src/tui/state/`
  页面级状态逻辑分拆层；`deploy.rs`、`packages.rs`、`home.rs`、`actions.rs`、`hosts.rs` 已经独立出去
- `mcbctl/src/tui/state/hosts/`
  `Users` / `Hosts` 页进一步拆成 `users.rs` 和 `runtime.rs`
- `mcbctl/src/tui/state/packages/`
  `Packages` 页进一步拆成 `browse.rs` 和 `mutate.rs`，不再把浏览、搜索、分组编辑和落盘全堆在一个文件里
- `pkgs/mcbctl/default.nix`
  把这些 Rust 二进制打成 Nix 包
- `home/modules/desktop.nix`
  把桌面用户需要的命令带进 Home Manager 环境

部署入口内部也开始收口成更清晰的层次：

- `mcbctl/src/bin/control/mcb-deploy.rs`
  部署向导的状态、流程和高层编排
- `mcbctl/src/bin/control/mcb-deploy/plan.rs`
  部署摘要、计划对象生成与命令预览
- `mcbctl/src/bin/control/mcb-deploy/wizard.rs`
  交互式向导步骤与回退流转
- `mcbctl/src/bin/control/mcb-deploy/execute.rs`
  `/etc/nixos` 备份、同步与 `nixos-rebuild` 执行
- `mcbctl/src/bin/control/mcb-deploy/selection.rs`
  主机/用户/管理员选择、模板解析和基础校验
- `mcbctl/src/bin/control/mcb-deploy/runtime.rs`
  per-user TUN、GPU、服务器运行时能力配置
- `mcbctl/src/bin/control/mcb-deploy/scaffold.rs`
  新 host / 新用户目录脚手架与 `local.nix` 生成
- `mcbctl/src/bin/control/mcb-deploy/source.rs`
  配置来源、本地仓库探测、远端拉取与镜像重试
- `mcbctl/src/bin/control/mcb-deploy/release.rs`
  release 版本、说明生成与 GitHub Release 发布

所以如果你改的是命令行为，重点看这里。
如果你改的是“这个命令怎么进入系统或用户环境”，就要连同 `pkgs/mcbctl/` 和 `home/modules/desktop.nix` 一起看。

## 当前比较关键的命令

- `mcbctl`
  TUI 控制台入口
- `mcbctl deploy`
  从同一入口直接转发到部署向导
- `mcbctl release`
  从同一入口直接转发到发布流程
- `mcb-tui`
  显式 TUI 别名；和 `mcbctl` 指向同一类控制台入口
- `mcb-deploy`
  直接部署 / release 流程
- `noctalia-gpu-mode`
  GPU specialisation 菜单与切换
- `lock-screen`
  锁屏入口
- `niri-run`
  Niri 会话辅助启动
- `noctalia-*`
  一组给状态栏和桌面交互用的命令
- `update-zed-source`
  更新 Zed 官网稳定版 pin
- `update-yesplaymusic-source`
  更新 YesPlayMusic 官网稳定版 pin
- `update-upstream-apps`
  一次更新上述两个 pin

## 在仓库里怎么用

如果你在仓库根目录，最推荐的方式是：

```bash
nix run .#mcbctl
```

如果你想从同一个入口直接进部署向导，也可以：

```bash
nix run .#mcbctl -- deploy
```

如果你想显式写 TUI 名称，也可以：

```bash
nix run .#mcb-tui
```

如果你要直接进部署向导：

```bash
nix run .#mcb-deploy
```

部署向导现在同时支持：

- 选择已有主机
- 从 `hosts/templates/laptop/` 新建桌面主机
- 从 `hosts/templates/server/` 新建服务器主机
- 如果 `/etc/nixos/hardware-configuration.nix` 缺失，会尝试自动生成这一份根目录硬件配置

当前 `Packages` 页已经可以：

- 切换用户
- 默认走 `nixpkgs` 搜索
- 用 `f` 在 `nixpkgs` 搜索和本地覆盖/已声明视图之间切换
- 本地覆盖层来自 `catalog/packages/*.toml` 与已声明的 managed 软件
- 读取 `catalog/groups.toml` 里的组标签、说明和排序
- 勾选支持的软件
- 按组写入 `home/users/<user>/managed/packages/*.nix`

当前 `Home` 页已经可以：

- 切换用户
- 读取 `catalog/home-options.toml` 里的字段标签、说明和顺序
- 调整 `Noctalia` 顶栏 profile
- 调整 `Zed` / `YesPlayMusic` 桌面入口
- 写入 `home/users/<user>/managed/settings/desktop.nix`

当前 `Users` / `Hosts` 页已经可以：

- 切换目标主机
- 读取当前主机生效中的 `mcb.*` 关键字段
- 编辑用户、管理员、主用户、主机角色、linger

当前 `Deploy` / `Actions` 页已经可以：

- `Deploy`
  - 本地仓库 / `/etc/nixos` 这类常见来源直接执行
  - 如果需要高级项或远端来源，自动退回完整 `mcb-deploy` 向导
  - 缺失 `/etc/nixos/hardware-configuration.nix` 时会走共享生成逻辑
- `Actions`
  - 直接执行 `flake check`
  - 直接执行 `flake update`
  - 检查 / 刷新上游 pin
  - 同步到 `/etc/nixos`
  - 重建当前主机
  - 退回完整 `mcb-deploy`
- 编辑代理、TUN、GPU、虚拟化相关结构化设置
- 把用户结构写回 `hosts/<host>/managed/users.nix`
- 把运行时能力写回 `hosts/<host>/managed/network.nix`、`gpu.nix`、`virtualization.nix`

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
cd mcbctl
cargo check
```

当前仓库已经把 Cargo 构建产物迁到 `/var/tmp/mcbctl-target`，避免把 `target/` 留在仓库里，干扰 `nix flake check path:$PWD` 这类开发态检查。

### 运行某个命令

```bash
cd mcbctl
cargo run --bin mcbctl
```

如果你要直接调试部署向导：

```bash
cd mcbctl
cargo run --bin mcb-deploy -- --help
```

或者：

```bash
cd mcbctl
cargo run --bin noctalia-gpu-mode
```

### 格式化

```bash
cd mcbctl
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
