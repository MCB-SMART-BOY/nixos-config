# Legacy compatibility entrypoint for non-flake workflows.
{ ... }:

{
  imports = [
    ./nixos/hosts/nixos-dev
  ];
}
