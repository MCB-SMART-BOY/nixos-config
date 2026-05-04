# Legacy 入口（非 Flake）：用于不使用 flakes 的部署方式。
# 逻辑与 flake.nix 保持一致，继续复用 hosts/ 与 home/ 结构。

{ config, ... }:

let
  # Legacy 方式：从 flake.lock 读取固定版本，保证可复现
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  mkGithubTarball =
    name:
    let
      node = lock.nodes.${name}.locked;
    in
    builtins.fetchTarball {
      url = "https://github.com/${node.owner}/${node.repo}/archive/${node.rev}.tar.gz";
      sha256 = node.narHash;
    };

  nixpkgsSrc = mkGithubTarball "nixpkgs";
  homeManagerSrc = mkGithubTarball "home-manager";
  noctaliaSrc = mkGithubTarball "noctalia";

  inputs = {
    nixpkgs = nixpkgsSrc;
    home-manager = homeManagerSrc;
    noctalia.homeModules.default = "${noctaliaSrc}/homeModules";
  };
in
{
  imports = [
    ./hosts/nixos
    (import "${homeManagerSrc}/nixos")
  ];

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.backupFileExtension = "bak";
  home-manager.extraSpecialArgs = { inherit inputs; };
  home-manager.users =
    let
      userList =
        if config.mcb.users != [ ] then
          config.mcb.users
        else
          [ config.mcb.user ];
      mkUser = name: {
        inherit name;
        value = import (./home/users + "/${name}");
      };
    in
    builtins.listToAttrs (map mkUser userList);
}
