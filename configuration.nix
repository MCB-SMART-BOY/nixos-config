# Legacy compatibility entrypoint for non-flake workflows.
{ vars, ... }:

let
  home-manager = builtins.fetchTarball "https://github.com/nix-community/home-manager/archive/release-25.11.tar.gz";
in
{
  imports = [
    ./host.nix
    (import "${home-manager}/nixos")
  ];

  home-manager.useGlobalPkgs = true;
  home-manager.useUserPackages = true;
  home-manager.extraSpecialArgs = { inherit vars; };
  home-manager.users.${vars.user} = import ./home/home.nix;
}
