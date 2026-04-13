# 主机配置（nixos）：指定用户、代理模式与主机级参数。
# 新手提示：这里是“主机层”的总入口，会导入 profiles + 硬件配置。

{
  config,
  lib,
  ...
}:

let
  hardwareModule =
    if builtins.pathExists ./hardware-configuration.nix then
      ./hardware-configuration.nix
    else
      ../_support/hardware-configuration-eval.nix;
in
{
  imports = [
    ../profiles/desktop.nix
    hardwareModule
  ]
  ++ lib.optional (builtins.pathExists ./managed/default.nix) ./managed/default.nix
  ++ lib.optional (builtins.pathExists ./local.auto.nix) ./local.auto.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  mcb = {
    # 主用户与用户列表（影响 Home Manager 与权限）
    user = "mcbnixos";
    users = [ "mcbnixos" ];
    # 代理与 TUN 相关参数
    tunInterface = "Meta";
    tunInterfaces = [
      "Meta"
      "Mihomo"
      "clash0"
    ];
    cpuVendor = "intel";
    proxyMode = "tun";
    proxyUrl = "";
    enableProxyDns = false;
    proxyDnsAddr = "127.0.0.1";
    proxyDnsPort = 53;
    # 每个用户独立 TUN（高级用法）
    perUserTun = {
      enable = true;
      # 默认关闭 per-user DNS 重定向，避免未配置监听端口时断网
      redirectDns = false;
      interfaces = {
        mcbnixos = "Meta";
      };
      dnsPorts = {
        mcbnixos = 1053;
      };
    };

    hardware.gpu = {
      # Hybrid 特化需要 busId（iGPU + NVIDIA）
      igpuVendor = "intel";
      prime = {
        intelBusId = "PCI:0:2:0";
        nvidiaBusId = "PCI:1:0:0";
      };
      nvidia.open = true;
      specialisations.enable = true;
      # 覆盖特化模式列表，加入 hybrid
      specialisations.modes = [
        "igpu"
        "hybrid"
        "dgpu"
      ];
    };
  };

  networking.hostName = "nixos";
  system.stateVersion = "25.11";
}
