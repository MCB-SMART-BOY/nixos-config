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
nix run .#run-rs
```

这个入口会带你把第一次最容易出错的部分都走完：

- 部署模式
- 配置来源
- 目标主机
- 用户列表
- 管理员用户
- per-user TUN
- GPU 覆盖
- 服务器预设

如果你已经非常熟悉仓库结构，也可以直接：

```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

## 2. 空机器起步时，先确认两件事

### 2.1 先生成硬件配置

```bash
sudo nixos-generate-config
```

常见位置是：

- `/etc/nixos/hardware-configuration.nix`

这套仓库也接受放在：

- `/etc/nixos/hosts/<hostname>/hardware-configuration.nix`

### 2.2 仓库要在当前目录可见

`run-rs` 需要从你当前所在的仓库目录读取 `flake.nix`、`hosts/`、`home/` 这些内容。
所以正确用法不是“随便在哪都能跑”，而是：

- 先 `cd` 到仓库根目录
- 再执行 `nix run .#run-rs`

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

- `scripts-rs` 能否成功构建
- 仓库里是否还残留旧的 Shell 脚本入口
- 是否还有 `writeShell*` 这类遗留定义

## 4. 给某个用户加软件，正确位置在哪里

答案很明确：

- `home/users/<user>/packages.nix`

这套仓库现在的思路是：

- 系统共享能力放系统层
- 某个用户自己的软件放用户层

例如你要给 `mcbnixos` 加软件，就改：

- `home/users/mcbnixos/packages.nix`

这样做的结果很直接：

- 其他用户不会平白多出一堆自己不用的软件
- Nix store 仍然共享构建产物，不会重复浪费
- 以后读配置时，你能看清“这是谁要的”

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
- `home/users/mcblaptopnixos/`
- `home/users/mcbservernixos/`

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

几个现实提醒：

- BIOS 如果已经锁成 `dGPU-only`，切回 `igpu` 或 `hybrid` 可能会黑屏
- `hybrid` 不是只写一个字符串，还需要正确的 busId
- `run-rs` 在向导里会优先尝试自动探测，再回退到现有配置

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

如果你正在改 `scripts-rs`，直接在目录里跑：

```bash
cd scripts-rs
cargo check
cargo run --bin run-rs
```

但对正常部署来说，更推荐的还是在仓库根目录用：

```bash
nix run .#run-rs
```

因为这样走的是和仓库实际接线一致的路径。
