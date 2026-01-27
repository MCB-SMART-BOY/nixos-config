# Legacy compatibility entrypoint for non-flake workflows.
{ config, ... }:

let
  home-manager = builtins.fetchTarball "https://github.com/nix-community/home-manager/archive/release-25.11.tar.gz";
in
{
  imports = [
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
        value = import (./home/users + "/${name}");
      };
    in
    builtins.listToAttrs (map mkUser userList);
}
