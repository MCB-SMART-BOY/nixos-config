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
  nixpkgsUnstableSrc = mkGithubTarball "nixpkgs-unstable";
  nixpkgs2411Src = mkGithubTarball "nixpkgs-24_11";
  homeManagerSrc = mkGithubTarball "home-manager";
  noctaliaSrc = mkGithubTarball "noctalia";

  inputs = {
    nixpkgs = nixpkgsSrc;
    nixpkgs-unstable = nixpkgsUnstableSrc;
    nixpkgs-24_11 = nixpkgs2411Src;
    home-manager = homeManagerSrc;
    # 非 flake 模式下提供 Noctalia Home Manager 模块入口
    noctalia.homeModules.default = "${noctaliaSrc}/homeModules";
  };
in
{
  imports = [
    # 这里固定使用 hosts/nixos 作为默认主机入口
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
        # Home Manager 用户入口：home/users/<name>/default.nix
        value = import (./home/users + "/${name}");
      };
    in
    builtins.listToAttrs (map mkUser userList);
}
