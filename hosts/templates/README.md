# 主机模板

这里放的是脚手架来源，不是会被 flake 直接部署的真实主机。

可用模板：

- `hosts/templates/laptop/`
- `hosts/templates/server/`

使用方式：

1. 复制模板到 `hosts/<hostname>/`
2. 修改 `default.nix` 中的主机名、主用户、用户列表等基础信息
3. 做真实部署前生成根目录 `hardware-configuration.nix`
4. 让 `flake.nix` 自动把它识别成真实主机

仓库 / CI 评估时，如果根目录没有真实硬件文件，入口会自动导入 `_support/hardware-configuration-eval.nix`。这只用于评估，不可替代真实部署所需的硬件配置。

模板里自带的 `managed/` 目录只代表初始受管结构，并且现在也使用统一的 `mcbctl-managed` 协议。真正运行时写回仍由 `mcbctl` 接管，手写长期逻辑应放到 `default.nix` 或 `local.nix`。
