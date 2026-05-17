# NixOS 配置 —— 两层架构，清晰边界

一套 NixOS + Home Manager 配置，分两层：

- `modules/` — **系统**（怎么工作，不写死机器特定值）
- `users/admin/` — **人**（用什么软件，怎么个性化）

克隆后 `./run.sh` 即可部署。机器特定值（GPU busId、用户名、主机名）在 `local.nix`（gitignored），硬件配置在 `hardware-configuration.nix`（`nixos-generate-config` 生成）。

---

## 目录

```
nixos-config/
├── flake.nix                # 入口：nixosConfigurations.host
├── local.nix                # 机器身份（gitignored）
├── hardware-configuration.nix  # nixos-generate-config 生成（gitignored）
│
├── modules/                 # 系统层（通用，不写死机器值）
│   ├── default.nix          #   聚合 + mkDefault 默认值
│   ├── options.nix          #   mcb.* 选项声明
│   ├── users.nix            #   账户创建
│   ├── boot.nix / networking.nix / nix.nix / …
│   └── services/            #   SSH / nix-ld / Flatpak
│
├── users/
│   └── admin/               # 用户层（管理员，唯一的预置用户）
│       ├── default.nix      #   入口
│       ├── packages.nix     #   软件清单
│       ├── git.nix          #   Git 身份
│       ├── desktop.nix      #   Noctalia / GPU wrapper / 桌面应用
│       ├── shell.nix        #   Zsh / Starship / Tmux
│       └── config/          #   dotfiles（niri / helix / fastfetch …）
│
├── overlays/                # nixpkgs 覆盖层
├── pkgs/                    # 自维护包（Zed / YesPlayMusic）
├── scripts/run/             # 部署工具
└── docs/
```

---

## 快速开始

```bash
git clone https://github.com/MCB-SMART-BOY/nixos-config.git
cd nixos-config
# 编辑 local.nix：填你的用户名、主机名、时区
# 运行 nixos-generate-config 生成 hardware-configuration.nix
./run.sh
```

日常重建：
```bash
sudo nixos-rebuild switch --flake .#host
```

---

## 加用户

```bash
./run.sh add-user bob --admin           # 管理员
./run.sh add-user alice                  # 普通用户
./run.sh add-user eve --copy-from admin  # 从 admin 复制配置模板
```

---

## 改什么去哪

| 想改的 | 去 |
|---|---|
| 主机名、用户名、时区 | `local.nix` |
| GPU / 磁盘 / 硬件 | `hardware-configuration.nix` |
| 系统包组 | `modules/packages.nix` |
| 防火墙 / DNS | `modules/networking.nix` |
| 内核 / sysctl | `modules/boot.nix` |
| 某个用户的软件 | `users/<user>/packages.nix` |
| dotfiles | `users/<user>/config/` |
| 部署流程 | `run.sh` + `scripts/run/` |

---

## 设计原则

1. **`modules/` 不写死机器值** — GPU busId、用户名、主机名全在 `local.nix`（gitignored）
2. **`users/` 只管人** — 系统包组在 `modules/`，个人软件在 `users/<user>/packages.nix`
3. **GPU 交给社区标准** — `hardware-configuration.nix` + `nixos-generate-config`，不自己抽象
4. **代理交给应用** — clash-verge-rev 的 GUI 管理 TUN/DNS，Nix 层只保留 NetworkManager + 防火墙
5. **预置用户只有一个** — `admin`，其他通过 `./run.sh add-user` 生成
