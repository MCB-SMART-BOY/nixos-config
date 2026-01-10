# Legacy compatibility entrypoint for non-flake workflows.
{ ... }:

{
  imports = [
    ./hosts/nixos-dev
  ];
}
