# Legacy 入口（非 Flake）：用于不使用 flakes 的部署方式。
# 逻辑与 flake.nix 保持一致，继续复用 hosts/ 与 home/ 结构。

{ config, ... }:

let
  # Legacy 方式下手动拉取 Home Manager（固定 release-25.11）
  home-manager = builtins.fetchTarball "https://github.com/nix-community/home-manager/archive/release-25.11.tar.gz";
in
{
  imports = [
    # 这里固定使用 hosts/nixos 作为默认主机入口
    ./hosts/nixos
    (import "${home-manager}/nixos")
  ];

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.backupFileExtension = "bak";
  home-manager.users =
    let
      userList =
        if config.mcb.users != [ ] then
          config.mcb.users
        else
          [ config.mcb.user ];
      mkUser = name: {
        inherit name;
        # Home Manager 用户入口：home/users/<name>/default.nix
        value = import (./home/users + "/${name}");
      };
    in
    builtins.listToAttrs (map mkUser userList);
}
