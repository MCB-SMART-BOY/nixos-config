# 使用说明

这页不是命令速查表，而是按真实维护场景写的。

如果你现在是下面这些情况，这页最有用：

- 第一次把这套配置落到机器上
- 已经在用这套仓库，想知道平时怎么更新
- 想给某个用户加软件，但不想影响其他用户
- 想追 Zed / YesPlayMusic 官网稳定版

## 1. 第一次部署，先用最稳的入口

在仓库根目录执行：

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config
nix run .#mcbctl
```

这个入口现在会直接进入 TUI 控制台。

如果你只想直接进入部署向导，用：

```bash
nix run .#mcb-deploy
```

它会带你把第一次最容易出错的部分都走完：

- 部署模式
- 配置来源
- 目标主机
- 用户列表
- 管理员用户
- per-user TUN
- GPU 覆盖
- 服务器预设

其中桌面主机的 GPU 会在部署阶段先自动识别：

- 单集显主机：直接按 `igpu` 写入，不再弹 GPU 切换问答
- 独显主机：直接按 `dgpu` 写入，不再弹 GPU 切换问答
- 多显卡主机：才继续提供 `hybrid/igpu/dgpu` 的 GPU 方案选择

当前这一版 TUI 已经接入：

- 部署任务模型
- `managed/` 机器管理区落点
- `Packages` 页面默认走 `nixpkgs` 搜索；`catalog/packages/*.toml` 现在只保留本地覆盖层 / 仓库内自维护包元数据
- `Packages` 页面可以为指定用户勾选软件，并按组写入 `home/users/<user>/managed/packages/*.nix`
  当前组标签、说明和排序来自 `catalog/groups.toml`
  常用按键：`←/→` 切用户，`f` 在 `nixpkgs` 搜索 / 本地覆盖视图之间切换，`/` 输入关键词，`Enter` 或 `r` 刷新搜索，`j/k` 选软件，`[`/`]` 切分类，`u/i` 切来源过滤，`g/G` 改目标组，`m/M` 整组移动，`,`/`.` 切组过滤，`z` 聚焦当前条目所在组，`Z` 清空组过滤，`n` 新建组，`R` 重命名当前组，`Space` 勾选，`s` 保存
  搜索范围：`id`、名称、分类、软件组、表达式、描述、来源、关键词、平台
- `Home` 页面可以为指定用户写入 `home/users/<user>/managed/settings/desktop.nix`
  当前字段标签、说明和顺序来自 `catalog/home-options.toml`
  当前支持：`Noctalia` 顶栏 profile、`Zed` 桌面入口、`YesPlayMusic` 桌面入口
  常用按键：`←/→` 切用户，`j/k` 选项，`h/l` 或 `Enter` 调整，`s` 保存
- `Users` 页面会读取当前目标主机的 `mcb.user`、`mcb.users`、`mcb.adminUsers`、`mcb.hostRole`、`mcb.userLinger`，并写入 `hosts/<host>/managed/users.nix`
  常用按键：`←/→` 切主机，`j/k` 选字段，`h/l` 调整枚举，`Enter` 编辑列表，`s` 保存
- `Hosts` 页面会读取当前目标主机的代理、TUN、GPU、虚拟化相关 `mcb.*` 设置，并分别写入 `hosts/<host>/managed/network.nix`、`gpu.nix`、`virtualization.nix`
  当前支持：`cacheProfile`、`proxyMode`、`proxyUrl`、`tunInterface`、`perUserTun.*`、`hardware.gpu.*`、`virtualisation.docker/libvirtd`
  常用按键：`←/→` 切主机，`j/k` 选字段，`h/l` 调整枚举/布尔，`Enter` 编辑文本或映射，`s` 保存
- `Deploy` 页面现在不只是预览：
  本地仓库 / `/etc/nixos` 这类常见来源已经可以直接执行，按 `x` 会临时退出 TUI，跑完同步与 `nixos-rebuild` 再返回
  如果来源是当前仓库且仓库不在 `/etc/nixos`，并且当前动作不是 rootless 下的 `build`，还会先同步到 `/etc/nixos`
  如果开启高级项，或者选择远端来源，则会自动退回完整 `mcb-deploy` 向导
- `Actions` 页面现在已经接通：
  当前支持 `flake check`、`flake update`、上游 pin 检查/刷新、同步到 `/etc/nixos`、重建当前主机、启动完整部署向导
  常用按键：`j/k` 选动作，`Enter` / `Space` / `x` 执行

下一阶段继续接的是：

- `Packages` 页继续往 channel / 搜索缓存扩展，让搜索结果不只停在 `nixpkgs`
- `Home` 页继续扩展更多结构化设置
- `Home` 页继续把 session/mime 等设置接进分片

如果你已经非常熟悉仓库结构，也可以直接：

```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

如果你要新增一台主机，`mcb-deploy` 现在也支持直接从模板生成：

- 新建桌面主机：来自 `hosts/templates/laptop/`
- 新建服务器主机：来自 `hosts/templates/server/`

生成时会先创建 `hosts/<hostname>/`，再继续写用户入口和 `local.nix` 覆盖。

## 2. 空机器起步时，先确认两件事

### 2.1 先生成硬件配置

```bash
sudo nixos-generate-config
```

常见位置是：

- `/etc/nixos/hardware-configuration.nix`

也就是和 `/etc/nixos/configuration.nix` 同级。

现在 `mcb-deploy` 和 `mcbctl` 的共享执行层在真正部署时，如果发现这份文件缺失，也会尝试自动生成到这里。

### 2.2 仓库要在当前目录可见

`mcbctl` / `mcb-deploy` 都需要从你当前所在的仓库目录读取 `flake.nix`、`hosts/`、`home/` 这些内容。
所以正确用法不是“随便在哪都能跑”，而是：

- 先 `cd` 到仓库根目录
- 再执行 `nix run .#mcbctl`

## 3. 日常维护，通常就这些动作

### 3.1 改完配置后直接切换

```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

### 3.2 先试跑，不急着落地

```bash
sudo nixos-rebuild test --flake .#<hostname>
```

### 3.3 只想确认能不能构建

```bash
sudo nixos-rebuild build --flake .#<hostname>
```

### 3.4 更新 flake 输入

```bash
nix flake update
sudo nixos-rebuild switch --flake .#<hostname>
```

### 3.5 做一次仓库健康检查

```bash
nix flake check
```

它现在会检查：

- `mcbctl` 能否成功构建
- 仓库里是否还残留旧的 Shell 脚本入口
- 是否还有 `writeShell*` 这类遗留定义

## 4. 给某个用户加软件，正确位置在哪里

答案很明确：

- `home/users/<user>/packages.nix`
- 或者用 `nix run .#mcbctl` 写入 `home/users/<user>/managed/packages/*.nix`

这套仓库现在的思路是：

- 系统共享能力放系统层
- 某个用户自己的软件放用户层

例如你要给 `mcbnixos` 加软件，就改：

- `home/users/mcbnixos/packages.nix`

这样做的结果很直接：

- 其他用户不会平白多出一堆自己不用的软件
- Nix store 仍然共享构建产物，不会重复浪费
- 以后读配置时，你能看清“这是谁要的”

如果你用 TUI 管理软件，建议这样理解边界：

- 手写、长期维护的软件声明：`home/users/<user>/packages.nix`
- TUI / 自动化工具写入的软件声明：`home/users/<user>/managed/packages/*.nix`

## 5. 新增用户，不是只新建一个目录就完事

如果你想让某个用户真正被系统接管，需要两层都考虑：

- 系统层：把用户加入主机配置里的 `mcb.users` / `mcb.adminUsers`
- 用户层：新增 `home/users/<user>/`

最小用户入口一般至少有：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/local.nix.example`

你可以参考：

- `home/users/mcbnixos/`
- `home/templates/users/laptop/`
- `home/templates/users/server/`

## 6. GPU specialisation 的常用用法

这套仓库支持：

- `igpu`
- `hybrid`
- `dgpu`

你可以从桌面按钮切，也可以手动：

```bash
sudo nixos-rebuild switch --specialisation gpu-igpu
sudo nixos-rebuild switch --specialisation gpu-hybrid
sudo nixos-rebuild switch --specialisation gpu-dgpu
```

桌面侧现在的交互是：

- 左键 GPU 按钮：打开模式菜单
- 右键 GPU 按钮：查看当前模式切换建议
- 文本显示：显示当前生效 GPU 模式；如果当前 specialisation 是 `base`，会在 tooltip 里额外说明默认模式

也可以手动查看当前会话建议：

```bash
noctalia-gpu-mode --session-note
```

如果你想先确认这台机器在项目里被判断成哪类 GPU 主机：

```bash
noctalia-gpu-mode --host-topology
```

当前会输出三类之一：

- `igpu-only`
- `multi-gpu`
- `dgpu-only`

其中 GPU 模式按钮和交互入口默认只在 `multi-gpu` 主机上启用；单集显或独显主机会自动隐藏这个入口，并退化成说明信息。

几个现实提醒：

- BIOS 如果已经锁成 `dGPU-only`，切回 `igpu` 或 `hybrid` 可能会黑屏
- `hybrid` 不是只写一个字符串，还需要正确的 busId
- `mcb-deploy` 在向导里会优先尝试自动探测，再回退到现有配置
- Waybar / Noctalia 会自动刷新，但已打开的图形应用通常不会迁移到新 GPU
- 涉及 `hybrid` 或 `dgpu` 的切换，更稳的做法仍然是重启图形应用；如果出现渲染异常，建议注销并重新登录图形会话

## 7. 代理、TUN、per-user 路由

最常见的主机级开关是：

```nix
mcb.proxyMode = "tun";   # 或 "http" / "off"
```

如果你要做“不同用户走不同接口 / 不同节点”，重点看：

- `mcb.perUserTun.enable`
- `mcb.perUserTun.interfaces`
- `mcb.perUserTun.dnsPorts`
- `mcb.perUserTun.redirectDns`

更稳的排障顺序通常是：

1. 先确认单实例 TUN 正常。
2. 再确认接口名和服务名一致。
3. 再上 per-user TUN。
4. 最后再碰 DNS 重定向。

细节排障在：

- [docs/NETWORK_CN.md](/home/mcbgaruda/projects/nixos-config/docs/NETWORK_CN.md)

## 8. 追新官网应用

Zed 和 YesPlayMusic 现在都走仓库自己的固定 pin，不是单纯等 nixpkgs。

更新它们时，用：

```bash
nix run .#update-upstream-apps
```

只检查是否落后，不改文件：

```bash
nix run .#update-upstream-apps -- --check
```

如果你只想更新其中一个：

```bash
nix run .#update-zed-source
nix run .#update-yesplaymusic-source
```

## 9. Rust 脚本怎么单独调试

如果你正在改 `mcbctl`，直接在目录里跑：

```bash
cd mcbctl
cargo check
cargo run --bin mcbctl
```

如果你要调试直接部署向导：

```bash
cd mcbctl
cargo run --bin mcb-deploy -- --help
```

但对正常部署来说，更推荐的还是在仓库根目录用：

```bash
nix run .#mcbctl
```

因为这样走的是和仓库实际接线一致的路径。
