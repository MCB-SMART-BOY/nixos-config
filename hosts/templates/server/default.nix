# 主机模板（server）：复制到 hosts/<hostname>/ 后再按需改主机名、用户与硬件配置。

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
    ../profiles/server.nix
    hardwareModule
  ]
  ++ lib.optional (builtins.pathExists ./managed/default.nix) ./managed/default.nix
  ++ lib.optional (builtins.pathExists ./local.auto.nix) ./local.auto.nix
  ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  mcb = {
    # 模板占位：复制后请改成真实用户名
    user = "your-user";
    users = [ "your-user" ];
    cpuVendor = "intel";
    proxyMode = "off";

    hardware.gpu = {
      # 服务器默认不提供 GPU 特化入口
      specialisations.enable = false;
    };
  };

  networking.hostName = "your-host";
  system.stateVersion = "25.11";
}
