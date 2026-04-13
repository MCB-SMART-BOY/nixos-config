# 主机模板

这里放的是脚手架来源，不是会被 flake 直接部署的真实主机。

可用模板：

- `hosts/templates/laptop/`
- `hosts/templates/server/`

使用方式：

1. 复制模板到 `hosts/<hostname>/`
2. 修改 `default.nix` 中的主机名、主用户、用户列表等基础信息
3. 生成根目录 `hardware-configuration.nix`
4. 让 `flake.nix` 自动把它识别成真实主机

模板里自带的 `managed/` 目录只代表初始受管结构。
真正运行时写回仍由 `mcbctl` 接管，手写长期逻辑应放到 `default.nix` 或 `local.nix`。
