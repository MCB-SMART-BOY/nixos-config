# 国内网络环境排障

这页不是网络理论课，而是一份“在中国大陆网络环境里，真的容易卡住的点”排障手册。

如果你遇到的是下面这些情况，就来这里：

- `nix flake update` 很慢，甚至超时
- `nixos-rebuild` 下载依赖失败
- Clash / TUN 明明开着，但 DNS 还是不通
- 多用户 TUN 配好了，看起来却像没生效

---

## 先有一个现实预期

在国内维护 NixOS，网络问题通常不是单点故障，而是几件事叠在一起：

- 上游源访问不稳定
- DNS 状态和代理模式不一致
- Clash 的 TUN、DNS、接口名、监听端口没对齐
- 你以为是 Nix 的问题，其实是系统代理没有接住

所以排障时，不要一上来同时改镜像、改代理、改 Clash、改 DNS。
一次只动一层，会快很多。

---

## 当前仓库的默认思路

### `mcb.proxyMode = "tun"`

默认倾向于把流量交给 TUN 路线处理。
如果你在这条路上走通了，体验通常最好；如果没走通，症状也会最明显。

### `mcb.proxyMode = "http"`

更像传统的系统 HTTP 代理模式。
它不如 TUN 全面，但排障简单、行为也更容易理解。

### `mcb.proxyMode = "off"`

完全不启用代理，走系统默认网络路径。

### `mcb.enableProxyDns = false`

这意味着：

- 即使处于 TUN 模式，也不强制把 DNS 固定到本地代理
- 你需要确认 Clash 或系统 DNS 的实际行为是不是和你的预期一致

这个设计更保守，也更适合多用户、复杂路由场景，但前提是你知道自己当前到底让谁在解析 DNS。

---

## 遇到下载慢或超时，先做这三件事

### 1. 看当前缓存策略

这套仓库优先建议你用：

- `mcb.nix.cacheProfile`

而不是直接去手改 Nix 配置细节。

常见选择：

```nix
# 默认更适合国内网络
mcb.nix.cacheProfile = "cn";

# 海外网络环境
# mcb.nix.cacheProfile = "global";

# 只走官方缓存
# mcb.nix.cacheProfile = "official-only";
```

如果你在公司内网或者自建缓存环境，也可以用：

```nix
mcb.nix = {
  cacheProfile = "custom";
  customSubstituters = [
    "https://cache.nixos.org"
    "https://your-cache.example.com"
  ];
  customTrustedPublicKeys = [
    "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
    "your-cache.example.com-1:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx="
  ];
};
```

### 2. 临时挂代理再试一次

```bash
export http_proxy="http://127.0.0.1:7890"
export https_proxy="http://127.0.0.1:7890"
export all_proxy="socks5://127.0.0.1:7890"

sudo -E nixos-rebuild switch
```

如果你只是偶发网络不通，这一步往往就够了。

### 3. 再决定是不是要改镜像

镜像不是越多越好。
如果当前问题只是代理没接好，先把代理修好，再谈镜像。

---

## Clash Verge / Mihomo 排障顺序

这部分最有效的方式不是“猜”，而是按顺序检查。

### 1. 服务到底活着没有

```bash
systemctl status clash-verge-service
systemctl status clash-verge-service@<user>
systemctl status mcb-tun-route@<user>
```

如果服务本身就没起来，后面看接口名、看 DNS 都是在浪费时间。

### 2. TUN 接口名和 Nix 配置是不是同一个东西

```bash
ip link
```

你在配置里写的可能是：

- `Meta`
- `Mihomo`
- `clash0`

而系统里实际起来的接口名，必须能对应上。
如果名字根本不一致，per-user 路由和 DNS 重定向都不会按你想的那样工作。

这时要回头检查：

- `mcb.tunInterface`
- `mcb.tunInterfaces`
- `mcb.perUserTun.interfaces`

### 3. DNS 实际走的是谁

```bash
cat /etc/resolv.conf
```

常见坏状态：

- 只有 `127.0.0.1`，但 Clash 的 DNS 根本没开
- Clash 在监听 `1053`，你却还以为是 `53`
- 系统以为自己走本地 DNS，实际本地 DNS 没有人在服务

如果你的 Clash DNS 不是标准 53 端口，记得在主机配置里同步：

```nix
mcb.proxyDnsPort = 1053;
```

---

## Noctalia 的代理状态为什么可能“看着不对”

Noctalia 里代理图标的状态来自脚本：

- `~/.local/bin/noctalia-proxy-status`

这个命令由 `scripts-rs` 打包，再通过 `home/users/<user>/scripts.nix` 链接进用户环境。

它默认会检查这些服务名：

- `clash-verge-service@<user>`
- `clash-verge-service`
- `mihomo`

如果你改了服务命名方式，或者自己用了别的服务模板，状态图标和实际运行状态可能就会脱节。
这时不要先怀疑 Noctalia，先看脚本假设和你当前服务名是不是还一致。

---

## 多用户 TUN，最容易错在哪里

当你想做“每个用户走不同接口 / 不同节点”时，要看的是：

- `mcb.perUserTun.enable`
- `mcb.perUserTun.interfaces`
- `mcb.perUserTun.dnsPorts`
- `mcb.perUserTun.redirectDns`

一个最小例子：

```nix
mcb.proxyMode = "tun";
mcb.enableProxyDns = false;
mcb.users = [ "mcbnixos" "mcblaptopnixos" ];
mcb.perUserTun.enable = true;
mcb.perUserTun.redirectDns = true;
mcb.perUserTun.interfaces = {
  mcbnixos = "Meta";
  mcblaptopnixos = "Mihomo";
};
mcb.perUserTun.dnsPorts = {
  mcbnixos = 1053;
  mcblaptopnixos = 1054;
};
```

这里有四个最容易踩的坑：

### 1. Clash 里的 `tun.device` 没和 Nix 配置一致

Nix 写一套，Clash 配一套，看起来“都写了”，实际上根本没连上。

### 2. 端口冲突

多个实例同时运行时，DNS 监听端口不能撞。

### 3. 以为 per-user TUN 会自动接管全局 DNS

它不是这么工作的。
尤其当你启用了按 UID 路由时，DNS 这块要单独确认。

### 4. `redirectDns` 开了，但 `dnsPorts` 不匹配

这种时候最典型的现象是：

- 路由好像有了
- 域名就是不通
- IP 直连却还能访问

---

## 什么时候该用 `http`，什么时候该用 `tun`

一个非常实用的判断标准：

### 优先用 `http`

当你现在最在意的是：

- 先把 Nix 下载恢复正常
- 先把系统更新跑通
- 先降低排障复杂度

### 优先用 `tun`

当你已经确认：

- Clash 服务稳定
- TUN 接口名一致
- DNS 行为也对
- 你需要更完整的透明代理体验

不要把 `tun` 当作一种“更高级所以应该先上”的状态。
对很多机器来说，先用 `http` 把系统维护链路打通，反而更务实。

---

## 常见报错，先怎么想

### `cannot download ... Connection timed out`

先检查：

1. 当前缓存策略是不是适合你现在的网络
2. 代理环境变量有没有真正传进去
3. 代理本身是不是能连上外网

如果只是偶发问题，不一定需要改配置，先临时代理重试一次。

### DNS 解析失败，域名不通但 IP 能通

优先看：

- `/etc/resolv.conf`
- Clash DNS 是否开启
- 端口是不是和 `mcb.proxyDnsPort` / `mcb.perUserTun.dnsPorts` 一致

### `hash mismatch`

先试：

```bash
sudo nix-collect-garbage
sudo nixos-rebuild switch
```

如果仍然失败，再去确认是不是上游源变了，或者你的 pin 需要更新。

---

## 一个更稳的排障节奏

如果你现在网络状态很乱，推荐按这个顺序来：

1. 先把 `mcb.proxyMode` 降到你能解释清楚的状态
2. 先确认单用户、单实例代理正常
3. 再确认 DNS 正常
4. 再确认 Nix 下载正常
5. 最后再叠 per-user TUN、多实例、DNS 重定向

国内网络环境下，真正省时间的方法不是“一次把理想方案全部配完”，而是每一步都确认它单独成立。
