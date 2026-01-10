{ pkgs, ... }:

let
  vars = import ../shared/vars.nix;
  user = vars.user;
in
{
  programs.zsh.enable = true;

  users.users.${user} = {
    isNormalUser = true;
    description = user;
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
    "d /home/${user}/.config/clash-verge 0750 ${user} users -"
    "d /var/lib/mihomo 0755 root root -"
  ];
}
