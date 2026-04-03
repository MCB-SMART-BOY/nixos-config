# 用户与主机身份相关选项：定义主用户、多用户、管理员与主机角色。

{ lib, ... }:

let
  inherit (lib) mkOption types;
in
{
  options.mcb = {
    user = mkOption {
      type = types.str;
      default = "user";
      description = "Primary system user name when a host does not declare one explicitly.";
    };

    users = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "All system users managed by this host (Home Manager will be enabled for each).";
    };

    adminUsers = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Users granted admin privileges (wheel). Defaults to mcb.user when unset in host config.";
    };

    hostRole = mkOption {
      type = types.enum [
        "desktop"
        "server"
      ];
      default = "desktop";
      description = "Host role used to derive default user group memberships.";
    };

    userLinger = mkOption {
      type = types.bool;
      default = false;
      description = "Enable user lingering for managed users.";
    };

    cpuVendor = mkOption {
      type = types.enum [
        "intel"
        "amd"
      ];
      default = "intel";
      description = "CPU vendor for kernel module selection.";
    };
  };
}
