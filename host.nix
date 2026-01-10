{ pkgs, ... }:

let
  vars = {
    user = "mcbnixos";
    tunInterface = "clash0";
    proxyUrl = "http://127.0.0.1:7890";
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
    "d /var/lib/mihomo 0755 root root -"
  ];
}
