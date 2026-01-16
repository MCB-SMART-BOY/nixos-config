{ pkgs, ... }:

let
  vars = {
    user = "mcbnixos";
    tunInterface = "clash0";
    # Leave empty to keep system proxy/DNS disabled by default.
    # When using Clash Verge, let it manage proxy/DNS on demand.
    proxyUrl = "";
  };
in
{
  _module.args.vars = vars;

  imports = [
    ./modules
    ./hardware-configuration.nix
  ];

  networking.hostName = "nixos";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  users.users.${vars.user} = {
    isNormalUser = true;
    description = vars.user;
    extraGroups = [
      "wheel"
      "networkmanager"
      "video"
      "audio"
      "docker"
      "libvirtd"
    ];
    shell = pkgs.zsh;
    linger = true;
  };

  systemd.tmpfiles.rules = [
    "d /home/${vars.user}/.config/clash-verge 0750 ${vars.user} users -"
    "d /home/${vars.user}/.config/clash-verge-rev 0750 ${vars.user} users -"
    "d /var/lib/mihomo 0755 root root -"
  ];
}
