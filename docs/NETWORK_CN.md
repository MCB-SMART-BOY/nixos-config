# 🇨🇳 中国境内网络问题解决方案

## 当前默认行为（基于本仓库配置）

- `host.nix` 中 `vars.proxyUrl` 非空时：
  - 启用系统代理（`networking.proxy`）
  - DNS 首选 `127.0.0.1`（需要本机 Clash 提供 DNS）
- `vars.proxyUrl` 为空时：
  - 不启用系统代理
  - DNS 直接走公网解析（不依赖 Clash）
- `vars.enableProxy = true` 时仍会启用代理相关服务/TUN DNS（即使 `proxyUrl` 为空）

## Clash Verge 排查清单

1. 服务是否正常：
   ```bash
   systemctl status clash-verge-service
   ```
2. TUN 网卡名是否匹配：
   ```bash
   ip link
   ```
   如果接口名不是 `clash0`，请修改 `host.nix` 的 `vars.tunInterface`。
3. DNS 是否由 Clash 提供：
   ```bash
   cat /etc/resolv.conf
   ```
   如果只有 `127.0.0.1` 但 Clash DNS 未启用，会导致解析失败。

## Waybar 代理指示

Waybar 的代理图标由 `home/scripts/waybar-proxy-status` 提供，默认检测 `clash-verge-service` / `mihomo`。
如果使用其他服务名，请修改脚本后重建。

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
2. 确认 `vars.tunInterface` 与实际 TUN 名一致
3. 重建后自动生效

## 常见错误

### cannot download ... Connection timed out

- 检查代理是否生效
- 切换到国内镜像或临时代理

### DNS 解析失败（ping 域名不通）

- Clash 未启用 DNS：把 `vars.proxyUrl` 置空，或在 Clash 开启 DNS
- 检查 `resolv.conf` 是否仅指向 `127.0.0.1`

### hash mismatch

```bash
sudo nix-collect-garbage
sudo nixos-rebuild switch
```

---

如仍无法联网，可尝试手机热点或在另一台机器构建后同步。
