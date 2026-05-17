# 国内网络环境提示

## 缓存镜像

中国大陆用户建议使用国内镜像加速 Nix 下载。在 `local.nix` 中设置：

```nix
mcb.nix.cacheProfile = "cn";
```

可选值：`cn`（国内镜像）/ `global`（国际源）/ `official-only`（仅官方）/ `custom`（自定义）。

---

## 代理

如果通过代理访问外网，在 `local.nix` 中设置：

```nix
mcb.proxyMode = "tun";   # TUN 模式（推荐，clash-verge-rev GUI 管理）
mcb.proxyMode = "http";  # HTTP 代理模式
mcb.proxyMode = "off";   # 不使用代理（默认）
```

TUN 模式下，clash-verge-rev 的 GUI 一键管理路由和 DNS。Nix 层只保留 NetworkManager + 防火墙，不干预策略路由。

HTTP 代理模式下，需同时设置代理地址：

```nix
mcb.proxyUrl = "http://127.0.0.1:7890";
```

---

## 遇到下载慢或超时

1. 确认缓存策略：`mcb.nix.cacheProfile` 是否设为 `"cn"`
2. 临时挂代理重试：
   ```bash
   export http_proxy="http://127.0.0.1:7890"
   export https_proxy="http://127.0.0.1:7890"
   sudo -E nixos-rebuild switch
   ```
3. 如果持续失败，先收集垃圾再试：
   ```bash
   sudo nix-collect-garbage
   sudo nixos-rebuild switch --flake .#host
   ```

---

## 排障优先级

1. 先确认单用户代理正常
2. 再确认 DNS 正常
3. 再确认 Nix 下载正常

不要同时改镜像、代理、DNS。
