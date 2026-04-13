# 网络与代理说明

这份文档只描述当前仓库真实存在的网络主线：缓存策略、`proxyMode`、TUN、per-user TUN、DNS，以及它们在 `mcbctl` 里的写回边界。

## 1. 缓存策略

支持的 `cacheProfile`：

- `cn`
- `global`
- `official-only`
- `custom`

`custom` 模式下，`mcbctl` 和 Nix 都要求这两项非空：

- `mcb.nix.customSubstituters`
- `mcb.nix.customTrustedPublicKeys`

## 2. 代理模式

只保留三种：

- `mcb.proxyMode = "tun"`
- `mcb.proxyMode = "http"`
- `mcb.proxyMode = "off"`

额外字段：

- `mcb.proxyUrl`
- `mcb.tunInterface`
- `mcb.tunInterfaces`
- `mcb.enableProxyDns`
- `mcb.proxyDnsAddr`
- `mcb.proxyDnsPort`

`http` 模式要求 `proxyUrl` 非空。

## 3. per-user TUN

结构化字段：

- `mcb.perUserTun.enable`
- `mcb.perUserTun.compatGlobalServiceSocket`
- `mcb.perUserTun.redirectDns`
- `mcb.perUserTun.interfaces`
- `mcb.perUserTun.dnsPorts`
- `mcb.perUserTun.tableBase`
- `mcb.perUserTun.priorityBase`

当前 Rust 校验会拦这些错误：

- `perUserTun.enable = true` 但 `proxyMode != "tun"`
- per-user TUN 和全局代理 DNS 同时开启
- 缺少用户接口映射
- 缺少 DNS 端口映射
- 接口名重复
- DNS 端口重复
- 映射里残留了已不在 `mcb.users` 中的用户
- `tableBase` / `priorityBase` 非正数

## 4. TUI 写回边界

`Hosts` 页现在直接拥有这些网络字段的读写闭环：

- 缓存策略和自定义缓存字段
- `proxyMode` / `proxyUrl`
- `tunInterface` / `tunInterfaces`
- `proxyDns*`
- per-user TUN 全套结构化字段

写回位置固定是：

- `hosts/<host>/managed/network.nix`

如果这个文件被手改到不再像受管文件，TUI 会拒绝覆盖；如果仓库来自旧树，先用 `mcbctl migrate-managed` 把旧格式受管文件升级掉。

## 5. 服务链

Nix 模块：

- `modules/nix.nix`
- `modules/networking.nix`
- `modules/services/core.nix`

Rust 命令：

- `clash-verge-prestart`
- `mcb-tun-route`
- `noctalia-proxy-status`

常见服务：

- `clash-verge-service`
- `clash-verge-service@<user>`
- `mcb-tun-route@<user>`
- `mihomo`

## 6. 常用排查

```bash
systemctl status clash-verge-service
systemctl status clash-verge-service@<user>
systemctl status mcb-tun-route@<user>
systemctl status mihomo
ip link
ip rule show
ip route show table all
resolvectl status
nix run .#mcbctl -- doctor
nix run .#noctalia-proxy-status
```

`doctor` 现在也会显示仓库根目录是否存在真实 `hardware-configuration.nix`，方便区分“网络问题”和“当前只是评估 fallback”。

## 7. 推荐顺序

如果网络状态很乱，按这个顺序收：

1. 先确认 `cacheProfile`
2. 再确认 `proxyMode`
3. 再确认 `proxyDns*`
4. 再确认单用户 TUN
5. 最后叠 per-user TUN 和 DNS 重定向

不要同时改缓存、代理、接口名、DNS 和 per-user 映射。
