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
      system = "x86_64-linux";
      hostEntries = builtins.readDir ./hosts;
      hostNames = builtins.filter (name:
        hostEntries.${name} == "directory" && name != "profiles"
      ) (builtins.attrNames hostEntries);
      mkHost = name:
        nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            (./hosts + "/${name}")
            home-manager.nixosModules.home-manager
            ({ config, ... }:
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
              {
                home-manager.useGlobalPkgs = true;
                home-manager.useUserPackages = true;
                home-manager.backupFileExtension = "bak";
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
