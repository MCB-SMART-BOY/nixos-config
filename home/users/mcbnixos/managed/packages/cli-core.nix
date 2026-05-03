# mcbctl-managed: package-group:cli-core
# mcbctl-checksum: 28e34aee608331121aa538a53d4c08e89db09225ce4d8dd8c1373a5c12d47859
{ lib, pkgs, ... }:

{
  home.packages = with pkgs; [
    ripgrep
    fd
    bat
    eza
    zoxide
    fzf
];
}
