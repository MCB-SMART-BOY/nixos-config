# 使用说明

这份文档不是“把所有命令抄一遍”的清单，而是按真实使用场景写的维护手册。

如果你现在处于下面这些情况，这页最适合你：

- 刚装完 NixOS，想把这套配置落上去
- 已经在用这套仓库，想知道日常更新怎么做
- 想给某个用户加软件，但不想影响其他用户
- 想改 GPU、代理、TUN，又不想在仓库里盲翻文件

---

## 1. 第一次部署，不要把事情搞复杂

先确认三件事：

- `hardware-configuration.nix` 已经生成
- 系统里能运行 `nixos-rebuild`
- 你已经把仓库拉到本地

推荐流程：

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config
./run.sh
```

`run.sh` 会把第一次部署里最容易出错的部分都收起来，换成向导让你选：

- 部署模式
- 配置来源
- 目标主机
- 用户列表
- 管理员用户
- per-user TUN
- GPU 覆盖
- 服务器预设

如果你以前习惯自己手写 `/etc/nixos`，这里也可以继续那样做；只是对这套仓库来说，第一次上手直接走向导通常更稳。

---

## 2. 如果你是从一台“空机器”开始

### 2.1 先生成硬件配置

```bash
sudo nixos-generate-config
```

最常见的位置是：

- `/etc/nixos/hardware-configuration.nix`

这套仓库也接受把它放在：

- `/etc/nixos/hosts/<hostname>/hardware-configuration.nix`

你不用一开始就纠结“哪种更优雅”。只要当前主机能被正确评估和重建，先跑起来最重要。

### 2.2 应用配置

如果你不通过 `run.sh`，那就手动执行：

```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

第一次切完以后，建议重启一次。尤其是你动了图形栈、驱动、代理或 specialisation 时，这一步能省掉很多“看起来怪怪的”问题。

---

## 3. 日常维护，其实就这几件事

### 3.1 修改配置后直接切换

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

如果你习惯在改完配置前先做一次健康检查，这套仓库也支持：

```bash
nix flake check
```

它现在不只看 Nix 求值，还会顺手检查：

- `run.sh` 和分层 shell 脚本
- ShellCheck
- `scripts-rs` 的 `cargo check`

---

## 4. 给某个用户加软件，正确姿势是什么

答案很明确：去写这个用户自己的文件。

- `home/users/<user>/packages.nix`

现在这套仓库的思路不是“把所有桌面软件都扔进系统层”，而是：

- 系统共享的东西放系统层
- 只属于某个用户的东西放用户层

例如你要给 `mcbnixos` 加 Zed、YesPlayMusic、浏览器、聊天软件，就写在：

- `home/users/mcbnixos/packages.nix`

这样做的结果是：

- 其他用户不会平白看到一堆自己根本不用的软件
- 构建产物仍然由 Nix store 共享，不会浪费
- 以后读配置时，不会搞不清这东西到底是谁要的

---

## 5. 新增用户，不需要一上来就复制整套配置

如果你通过 `./run.sh` 新增了一个仓库里还不存在的用户，脚本会帮你生成最小模板：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/local.nix.example`

默认行为是保守的：

- 先给你一个能工作的骨架
- 不主动复制别人的整套 `config/`、`assets/`、`scripts/`

如果你明确想复用模板用户的这些目录，可以在运行前加：

```bash
RUN_SH_COPY_USER_TEMPLATE=true ./run.sh
```

这更适合“批量生成一组风格接近的新用户”的场景。

---

## 6. 主机层和用户层，别混

你可以这样记：

### 主机层回答的是“这台机器应该是什么样”

看这里：

- `hosts/<hostname>/default.nix`
- `hosts/profiles/desktop.nix`
- `hosts/profiles/server.nix`

主机层负责的是：

- 主机名
- 默认用户 / 用户列表 / 管理员用户
- 系统服务
- 系统共享包组
- GPU、网络、缓存、虚拟化这些机器级能力

### 用户层回答的是“这个人想怎么用这台机器”

看这里：

- `home/users/<user>/default.nix`
- `home/users/<user>/packages.nix`
- `home/users/<user>/config/`

用户层负责的是：

- 个人软件清单
- Niri / Noctalia / 终端 / 编辑器配置
- 主题、快捷键、界面细节
- 只对这个用户生效的个性化设置

---

## 7. GPU specialisation，先理解再切

这套仓库支持：

- `igpu`
- `hybrid`
- `dgpu`

你可以通过 Noctalia 顶栏里的 GPU 项切换，也可以手动切：

```bash
sudo nixos-rebuild switch --specialisation gpu-igpu
sudo nixos-rebuild switch --specialisation gpu-hybrid
sudo nixos-rebuild switch --specialisation gpu-dgpu
```

几个关键提醒：

- 如果 BIOS 已经锁成 `dGPU-only`，切回 `igpu` 或 `hybrid` 可能直接黑屏
- `hybrid` 不是只写一个字符串就完事，通常还需要正确的 busId
- 向导在配置 `hybrid` 时会优先尝试自动探测 busId，探测不到才回退到主机配置

如果你在这里出问题，不要先猜。先去看：

- [docs/DETAILS.md](/home/mcbgaruda/projects/nixos-config/docs/DETAILS.md)

---

## 8. 代理、TUN、per-user 路由

常见模式是这三个：

```nix
mcb.proxyMode = "tun";   # 或 "http" / "off"
```

如果你要做“不同用户走不同节点 / 不同 TUN 接口”，看的是：

- `mcb.perUserTun.*`

这部分的经验结论只有一句：

不要一边改 Clash 配置，一边改 Nix 配置，还同时改 DNS 端口，然后指望第一次就对。

更稳的做法是：

1. 先确认单实例 TUN 正常
2. 再确认接口名一致
3. 再上 per-user TUN
4. 最后再折腾 DNS 重定向

详细排障页在这里：

- [docs/NETWORK_CN.md](/home/mcbgaruda/projects/nixos-config/docs/NETWORK_CN.md)

---

## 9. 追新官网应用

Zed 和 YesPlayMusic 在这套仓库里不是简单依赖 nixpkgs，而是做了固定 pin。

更新它们时，用：

```bash
./pkgs/scripts/update-upstream-apps.sh
```

如果你只是想知道“上游有没有更新”，但不想立刻改文件：

```bash
./pkgs/scripts/update-upstream-apps.sh --check
```

更新完别忘了：

```bash
sudo nixos-rebuild switch --flake .#<hostname>
```

---

## 10. 回滚，别等出事了再想起来

系统级回滚有两种常见方式：

- 重启后在 boot 菜单里选旧 generation
- 直接执行：

```bash
sudo nixos-rebuild switch --rollback
```

如果你准备做较大的结构调整，建议额外做两件事：

```bash
git status
git tag before-big-change
```

你以后会感谢今天这个多出来的 10 秒钟。

---

## 11. 推荐维护节奏

这套仓库比较舒服的维护方式通常是：

1. 先改一个小目标，不要一口气改五层
2. 先 `nix flake check`
3. 再 `nixos-rebuild test`
4. 没问题再 `switch`
5. 大改后重启一次

如果你发现自己越来越依赖“试一下看会不会炸”，那通常不是你手速不够快，而是这次改动边界没收干净。

先回到目录分工，把问题缩回一层，再继续做，会轻松很多。
