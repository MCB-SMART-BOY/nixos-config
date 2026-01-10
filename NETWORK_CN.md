# 🇨🇳 中国境内网络问题解决方案

## 方案 1：使用国内镜像（已配置）

配置文件已包含以下镜像源：
- 中科大: `https://mirrors.ustc.edu.cn/nix-channels/store`
- 清华: `https://mirrors.tuna.tsinghua.edu.cn/nix-channels/store`
- 上海交大: `https://mirror.sjtu.edu.cn/nix-channels/store`

## 方案 2：临时使用代理

### 方法 A：命令行临时代理

```bash
# 设置代理环境变量
export http_proxy="http://127.0.0.1:7890"
export https_proxy="http://127.0.0.1:7890"
export all_proxy="socks5://127.0.0.1:7890"

# 然后运行
sudo -E nixos-rebuild switch
```

### 方法 B：修改配置文件永久代理

如果使用本仓库结构，建议调整 `nixos/modules/networking.nix` 或 `nixos/modules/services.nix`；传统 `/etc/nixos` 可以直接编辑 `configuration.nix` 并修改端口：

```nix
networking.proxy = {
  default = "http://127.0.0.1:7890";
  noProxy = "127.0.0.1,localhost,internal.domain";
};
```

或者使用环境变量方式：

```nix
environment.variables = {
  http_proxy = "http://127.0.0.1:7890";
  https_proxy = "http://127.0.0.1:7890";
  all_proxy = "socks5://127.0.0.1:7890";
  no_proxy = "localhost,127.0.0.1,::1";
};
```

## 方案 3：更换 Channel 到国内镜像

```bash
# 删除官方 channel
sudo nix-channel --remove nixos

# 添加中科大镜像
sudo nix-channel --add https://mirrors.ustc.edu.cn/nix-channels/nixos-25.11 nixos

# 或者清华镜像
# sudo nix-channel --add https://mirrors.tuna.tsinghua.edu.cn/nix-channels/nixos-25.11 nixos

# 更新
sudo nix-channel --update
```

## 方案 4：使用 Clash/V2Ray 透明代理

如果你有 Clash 或 V2Ray：

1. 开启 Clash TUN 模式或系统代理
2. 确保代理软件正常运行
3. 直接运行 `sudo nixos-rebuild switch`

## 方案 5：离线安装（高级）

如果完全无法联网，可以在能联网的机器上预下载：

```bash
# 在能联网的机器上
nix-store --export $(nix-store -qR /run/current-system) > system.nar

# 复制到目标机器后导入
nix-store --import < system.nar
```

## 常见错误

### 错误：cannot download ... Connection timed out

```bash
# 1. 检查镜像是否可访问
curl -I https://mirrors.ustc.edu.cn/nix-channels/store/nix-cache-info

# 2. 如果不行，尝试其他镜像或代理
```

### 错误：hash mismatch

```bash
# 清理缓存重试
sudo nix-collect-garbage
sudo nixos-rebuild switch
```

### 错误：SSL certificate problem

```bash
# 临时禁用 SSL 验证（不推荐，仅调试用）
export NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
```

## 推荐的代理软件

- **Clash Verge**: 支持 Linux，有 TUN 模式
- **v2rayA**: Web 界面管理
- **sing-box**: 轻量级

安装 Clash（在 NixOS 中）：

```nix
environment.systemPackages = with pkgs; [
  clash-verge-rev  # 或 clash-meta
];
```

---

如果以上方案都不行，可以考虑：
1. 使用手机热点（有些运营商限制较少）
2. 在 VPS 上构建后同步
3. 使用 NixOS ISO 的离线包
