{
  description = "My NixOS configuration (just to lock the version)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # git-hooks = {
    #   url = "github:cachix/git-hooks.nix";
    #   inputs.nixpkgs.follow = "nixpkgs";
    # }
  };

  outputs = inputs@ { self, ... }:
    import ./flake { inherit inputs self; };
}
