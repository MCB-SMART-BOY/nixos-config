{
  description = "My NixOS configuration (just to lock the version)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ self, ... }: import ./flake { inherit inputs self; };
}
