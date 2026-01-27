{ config, lib, pkgs, ... }:

let
  hardwareFile =
    if builtins.pathExists ./hardware-configuration.nix then
      ./hardware-configuration.nix
    else if builtins.pathExists ../../hardware-configuration.nix then
      ../../hardware-configuration.nix
    else
      null;
  allUsers =
    if config.mcb.users != [ ] then
      config.mcb.users
    else
      [ config.mcb.user ];
in
{
  imports =
    [ ../profiles/desktop.nix ]
    ++ lib.optional (hardwareFile != null) hardwareFile
    ++ lib.optional (builtins.pathExists ./local.nix) ./local.nix;

  mcb = {
    user = "mcblaptopnixos";
    users = [ "mcblaptopnixos" ];
    tunInterface = "Meta";
    tunInterfaces = [
      "Meta"
      "Mihomo"
      "clash0"
    ];
    cpuVendor = "intel";
    proxyMode = "tun";
    proxyUrl = "";
    enableProxyDns = false;
    proxyDnsAddr = "127.0.0.1";
    proxyDnsPort = 53;
    perUserTun = {
      enable = true;
      redirectDns = true;
      interfaces = {
        mcblaptopnixos = "Meta";
      };
      dnsPorts = {
        mcblaptopnixos = 1053;
      };
    };
  };

  networking.hostName = "laptop";
  system.stateVersion = "25.11";

  programs.zsh.enable = true;

  users.users = lib.genAttrs allUsers (name: {
    isNormalUser = true;
    description = name;
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
  });

  systemd.tmpfiles.rules =
    (lib.concatLists (map (name: [
      "d /home/${name}/.config/clash-verge 2775 ${name} users -"
      "d /home/${name}/.config/clash-verge-rev 2775 ${name} users -"
    ]) allUsers))
    ++ [
      "d /var/lib/mihomo 0755 root root -"
    ];
}
