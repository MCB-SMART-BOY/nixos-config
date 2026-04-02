# 主机模板（laptop）：复制到 hosts/<hostname>/ 后再按需改主机名、用户与硬件配置。

{
  config,
  lib,
  ...
}:

let
  hardwareFile =
    if builtins.pathExists ../../hardware-configuration.nix then
      ../../hardware-configuration.nix
    else
      null;
in
{
  imports = [
    ../profiles/desktop.nix
  ]
  ++ lib.optional (hardwareFile != null) hardwareFile
  ++ lib.optional (builtins.pathExists ./managed/default.nix) ./managed/default.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  # 允许在仓库/CI 环境中评估 flake（此时通常没有根目录 hardware-configuration.nix）
  fileSystems = lib.mkIf (hardwareFile == null) {
    "/" = {
      # 评估占位值：若用于真实部署会快速失败，避免误挂载错误磁盘。
      device = "/dev/disk/by-label/__MISSING_HARDWARE_CONFIGURATION__";
      fsType = "ext4";
    };
  };
  warnings = lib.optional (hardwareFile == null) ''
    这是主机模板；实际部署前请在仓库根目录补齐 hardware-configuration.nix（对应 /etc/nixos/hardware-configuration.nix）。
  '';

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
