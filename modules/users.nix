# 统一用户与权限模型：根据 mcb.user/mcb.users/mcb.adminUsers 自动创建账户与组。

{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.mcb;
  allUsers = if cfg.users != [ ] then cfg.users else [ cfg.user ];
  adminUsers = if cfg.adminUsers != [ ] then cfg.adminUsers else [ cfg.user ];
  desktopGroups =
    if cfg.hostRole == "desktop" then
      [
        "video"
        "audio"
      ]
    else
      [ ];
in
{
  config = {
    # 统一启用 fish，并将其加入 /etc/shells，供用户默认登录 shell 使用。
    programs.fish.enable = true;

    # 为每个用户创建私有组，避免共享 users 组导致跨用户目录权限扩大。
    users.groups = lib.genAttrs allUsers (_: { });

    # 自动创建系统用户并按需加入管理员/虚拟化相关组。
    users.users = lib.genAttrs allUsers (name: {
      isNormalUser = true;
      description = name;
      group = name;
      extraGroups =
        (lib.optionals (lib.elem name adminUsers) [ "wheel" ])
        ++ [
          "users"
          "networkmanager"
        ]
        ++ desktopGroups
        ++ lib.optionals config.virtualisation.docker.enable [ "docker" ]
        ++ lib.optionals config.virtualisation.libvirtd.enable [ "libvirtd" ];
      shell = pkgs.fish;
      linger = cfg.userLinger;
    });
  };
}
