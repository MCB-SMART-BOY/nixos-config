# 主机模板（laptop）：复制到 hosts/<hostname>/ 后再按需改主机名、用户与硬件配置。

{
  config,
  lib,
  ...
}:

let
  hardwareModule =
    if builtins.pathExists ../../hardware-configuration.nix then
      ../../hardware-configuration.nix
    else
      ../_support/hardware-configuration-eval.nix;
in
{
  imports = [
    ../profiles/desktop.nix
    hardwareModule
  ]
  ++ lib.optional (builtins.pathExists ./managed/default.nix) ./managed/default.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  mcb = {
    # 模板占位：复制后请改成真实用户名
    user = "your-user";
    users = [ "your-user" ];
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
    perUserTun = {
      enable = true;
      redirectDns = true;
      interfaces = {
        your-user = "Meta";
      };
      dnsPorts = {
        your-user = 1053;
      };
    };

    hardware.gpu = {
      # 模板本身不再写死具体机器的 busId。
      # 实际首次部署时，mcb-deploy 会根据当前主机自动识别 GPU 拓扑并写入 hosts/<host>/local.nix。
      specialisations.enable = false;
    };
  };

  networking.hostName = "your-host";
  system.stateVersion = "25.11";
}
