# 🇨🇳 中国境内网络问题解决方案

## 当前默认行为（基于本仓库配置）

- `mcb.proxyMode = "tun"`：
  - 启用 TUN 相关服务
  - DNS 强制指向本地地址（默认 `127.0.0.1:53`，可通过 `proxyDnsAddr`/`proxyDnsPort` 调整）
  - 不配置公网 fallback DNS（避免泄漏）
- `mcb.proxyMode = "http"`：
  - 启用系统 HTTP 代理（`networking.proxy`）
  - DNS 走系统默认解析（可使用 fallback）
- `mcb.proxyMode = "off"`：
  - 不启用代理
  - DNS 走系统默认解析
- `mcb.enableProxyDns = false` 时，即使处于 TUN 模式也不会强制本地 DNS

## Clash Verge 排查清单

1. 服务是否正常：
   ```bash
   systemctl status clash-verge-service
   # 多用户 TUN 模式：
   systemctl status clash-verge-service@<user>
   systemctl status mcb-tun-route@<user>
   ```
2. TUN 网卡名是否匹配：
   ```bash
   ip link
   ```
   如果接口名不是 `clash0`，请修改 `hosts/<hostname>/default.nix` 的 `mcb.tunInterface`，或使用 `mcb.tunInterfaces` 配置多个候选名称。
3. DNS 是否由 Clash 提供：
   ```bash
   cat /etc/resolv.conf
   ```
   如果只有 `127.0.0.1` 但 Clash DNS 未启用，会导致解析失败。
   如果 Clash DNS 监听的是其他端口（如 1053），请在 `hosts/<hostname>/default.nix` 设置 `mcb.proxyDnsPort = 1053;`。

## Waybar 代理指示

Waybar 的代理图标由 `home/users/<user>/scripts/waybar-proxy-status` 提供，默认检测 `clash-verge-service@<user>` / `clash-verge-service` / `mihomo`。
如果使用其他服务名，请修改脚本后重建。

## 多用户 TUN（按用户路由）

当需要“每个用户走不同的 TUN/节点”时，可开启 per-user 方案：

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

说明：
- 每个用户的 Clash 配置里，`tun.device` 必须与上面的接口名一致
- per-user 方案通过 `ip rule` 按 UID 路由，不支持全局强制 DNS
 - 若启用 `redirectDns`，会通过 iptables OUTPUT 按 UID 重定向 DNS，请确保 Clash 的 DNS 监听端口与 `dnsPorts` 一致
- 如果只启用一个用户，依然可用，但没有“多用户分流”的效果
- 多实例同时运行时，需确保各用户的 `mixed-port` / `socks-port` / `http-port` / `external-controller` / `dns.listen` 端口不冲突

## 方案 1：使用国内镜像（可选）

在 `modules/nix.nix` 添加以下配置：

```nix
nix.settings = {
  substituters = [
    "https://mirrors.ustc.edu.cn/nix-channels/store"
    "https://mirrors.tuna.tsinghua.edu.cn/nix-channels/store"
    "https://mirror.sjtu.edu.cn/nix-channels/store"
  ];
  trusted-public-keys = [
    "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
  ];
};
```

## 方案 2：临时使用代理（一次性）

```bash
export http_proxy="http://127.0.0.1:7890"
export https_proxy="http://127.0.0.1:7890"
export all_proxy="socks5://127.0.0.1:7890"

sudo -E nixos-rebuild switch
```

## 方案 3：非 Flake 的 Channel 镜像（可选）

```bash
sudo nix-channel --remove nixos
sudo nix-channel --add https://mirrors.ustc.edu.cn/nix-channels/nixos-25.11 nixos
sudo nix-channel --update
```

## 方案 4：透明代理（Clash/V2Ray）

1. 开启 Clash TUN + DNS
2. 确认 `mcb.tunInterface` 与实际 TUN 名一致
3. 重建后自动生效

## 常见错误

### cannot download ... Connection timed out

- 检查代理是否生效
- 切换到国内镜像或临时代理

### DNS 解析失败（ping 域名不通）

- Clash 未启用 DNS：把 `mcb.proxyMode` 改为 `"http"` 或 `"off"`，或在 Clash 开启 DNS
- 检查 `resolv.conf` 是否仅指向 `127.0.0.1`

### hash mismatch

```bash
sudo nix-collect-garbage
sudo nixos-rebuild switch
```

---

如仍无法联网，可尝试手机热点或在另一台机器构建后同步。
