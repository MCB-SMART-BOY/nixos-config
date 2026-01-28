# 入口文件（Flake）：定义依赖、主机列表与 Home Manager 组合方式。
# 新增/删除主机时，先看这里如何扫描 hosts/。

{
  description = "NixOS + Home Manager configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, home-manager, ... }:
    let
      # 当前系统架构（多数桌面是 x86_64-linux）
      system = "x86_64-linux";
      # 自动读取 hosts/ 下的主机目录（排除 profiles）
      hostEntries = builtins.readDir ./hosts;
      hostNames = builtins.filter (name:
        hostEntries.${name} == "directory" && name != "profiles"
      ) (builtins.attrNames hostEntries);
      # 为每个主机构造 nixosSystem，并注入 Home Manager
      mkHost = name:
        nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            (./hosts + "/${name}")
            home-manager.nixosModules.home-manager
            ({ config, ... }:
              let
                # 从 mcb.users 收集所有用户，否则使用单一 mcb.user
                userList =
                  if config.mcb.users != [ ] then
                    config.mcb.users
                  else
                    [ config.mcb.user ];
                # 将每个用户映射到 home/users/<name>
                mkUser = name: {
                  inherit name;
                  value = import (./home/users + "/${name}");
                };
              in
              {
                # Home Manager 与系统共享同一套 pkgs
                home-manager.useGlobalPkgs = true;
                home-manager.useUserPackages = true;
                home-manager.backupFileExtension = "bak";
                # 批量启用 Home Manager 用户
                home-manager.users = builtins.listToAttrs (map mkUser userList);
              })
          ];
        };
    in
    {
      nixosConfigurations =
        builtins.listToAttrs (map (name: { inherit name; value = mkHost name; }) hostNames);

      formatter.${system} = nixpkgs.legacyPackages.${system}.nixfmt;
    };
}
