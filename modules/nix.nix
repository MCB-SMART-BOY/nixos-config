# Nix 本体配置：flakes、缓存、GC、zram 等系统级设置。
# 这些设置影响构建性能与磁盘占用。

{ config, lib, ... }:

let
  cfg = config.mcb.nix;
  cacheNixos = "https://cache.nixos.org";
  cacheCommunity = "https://nix-community.cachix.org";
  cacheWayland = "https://nixpkgs-wayland.cachix.org";
  cnMirrors = [
    "https://mirrors.ustc.edu.cn/nix-channels/store"
    "https://mirrors.tuna.tsinghua.edu.cn/nix-channels/store"
    "https://mirror.sjtu.edu.cn/nix-channels/store"
  ];
  keyNixos = "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=";
  keyCommunity = "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=";
  keyWayland = "nixpkgs-wayland.cachix.org-1:3lwxaILxMRkVhehr5StQprHdEo4IrE8sRho9R9HOLYA=";
  profileSubstituters =
    if cfg.cacheProfile == "cn" then
      [
        cacheNixos
      ]
      ++ cnMirrors
      ++ [
        cacheCommunity
        cacheWayland
      ]
    else if cfg.cacheProfile == "global" then
      [
        cacheNixos
        cacheCommunity
        cacheWayland
      ]
    else if cfg.cacheProfile == "official-only" then
      [ cacheNixos ]
    else
      cfg.customSubstituters;
  profileTrustedKeys =
    if cfg.cacheProfile == "official-only" then
      [ keyNixos ]
    else if cfg.cacheProfile == "custom" then
      cfg.customTrustedPublicKeys
    else
      [
        keyNixos
        keyCommunity
        keyWayland
      ];
in
{
  assertions = lib.optionals (cfg.cacheProfile == "custom") [
    {
      assertion = cfg.customSubstituters != [ ];
      message = "mcb.nix.cacheProfile = \"custom\" requires non-empty mcb.nix.customSubstituters.";
    }
    {
      assertion = cfg.customTrustedPublicKeys != [ ];
      message = "mcb.nix.cacheProfile = \"custom\" requires non-empty mcb.nix.customTrustedPublicKeys.";
    }
  ];

  nix = {
    settings = {
      # 启用新命令与 Flakes
      experimental-features = [
        "nix-command"
        "flakes"
      ];
      # 二进制缓存源（可按 mcb.nix.cacheProfile 切换）
      substituters = profileSubstituters;
      trusted-public-keys = profileTrustedKeys;
      # 并行编译设置（按机器性能调整）
      max-jobs = lib.mkDefault "auto";
      cores = lib.mkDefault 0;
      auto-optimise-store = true;
    };
    gc = {
      # 自动垃圾回收，避免 /nix/store 过大
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 7d";
    };
  };

  nixpkgs.config = {
    # 允许非自由软件（如 Chrome）
    allowUnfree = true;
    # 仅在显式开启 mcb.packages.enableInsecureTools 时允许不安全包。
    permittedInsecurePackages = lib.optionals config.mcb.packages.enableInsecureTools [
      "ventoy-1.1.07"
    ];
  };

  zramSwap = {
    # 启用 zram，提高低内存场景可用性
    enable = true;
  };
}
