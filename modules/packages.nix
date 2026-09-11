{ lib, pkgs, config, ... }:

let
  baseRuntime = with pkgs; [
    bash
    coreutils
    findutils
    gawk
    gnugrep
    iproute2
    procps
    util-linux
    systemd
    vulkan-loader
  ];

  shellTools = with pkgs; [
    git lazygit
    wget curl
    eza fd fzf ripgrep bat
    delta zoxide
    starship direnv
    oh-my-zsh zsh-autosuggestions zsh-syntax-highlighting zsh-completions
    fish oh-my-fish
    btop bottom fastfetch
    duf gdu dust procs jq yq age sops
    lm_sensors yazi
    pciutils usbutils virt-viewer
  ];

  modernCli = with pkgs; [
    uutils-coreutils
    sd choose moreutils
    ripgrep-all difftastic
    tealdeer ouch dua xh
    jaq jless fx glow watchexec
  ];

  waylandTools = with pkgs; [
    wl-clipboard
    grim slurp libnotify
    brightnessctl
  ];

  entertainment = with pkgs; [
    ffmpeg
    linux-wallpaperengine
    mangohud
    protonup-qt
    lutris
  ];

  xorgCompat = with pkgs; [
    xwayland
    xhost
  ];

  niriRuntime = with pkgs; [
    xwayland-satellite
  ];

  geekTools = with pkgs; [
    strace ltrace
    gdb lldb
    patchelf file
    htop iotop iftop
    sysstat lsof mtr
    nmap tcpdump traceroute
    socat iperf3 ethtool
    hyperfine tokei tree
    zip unzip p7zip
    rsync rclone
    just entr ncdu binwalk radare2
    wireshark vulkan-tools gh hexyl doggo
    gping trippy bandwhich fq ast-grep sad
  ];

  groups = lib.concatLists [
    baseRuntime
    shellTools
    modernCli
    waylandTools
    entertainment
    xorgCompat
    niriRuntime
    geekTools
  ];

in {
  environment.systemPackages = groups;
}
