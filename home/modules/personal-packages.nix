# 用户个人应用包：按用户独立声明，不注入系统级 systemPackages。
# 适合多用户主机按需安装 GUI/开发应用，同时复用同一份 Nix store 构建产物。

{
  config,
  lib,
  pkgs,
  inputs,
  ...
}:

let
  cfg = config.mcb.personalPackages;

  unstablePkgs = import inputs.nixpkgs-unstable {
    system = pkgs.stdenv.hostPlatform.system;
    config = pkgs.config;
  };

  zedStable =
    let
      eval =
        if pkgs ? zed-editor-fhs then
          builtins.tryEval pkgs.zed-editor-fhs
        else
          {
            success = false;
            value = null;
          };
    in
    if eval.success then eval.value else null;

  zedOfficial =
    let
      eval = builtins.tryEval (pkgs.callPackage ../../pkgs/zed { });
    in
    if eval.success then eval.value else null;

  zedUnstable =
    let
      eval =
        if unstablePkgs ? zed-editor-fhs then
          builtins.tryEval unstablePkgs.zed-editor-fhs
        else
          {
            success = false;
            value = null;
          };
    in
    if eval.success then eval.value else null;

  zedPkg =
    if cfg.zedChannel == "official-stable" then
      zedOfficial
    else if cfg.zedChannel == "unstable" then
      (if zedUnstable != null then zedUnstable else zedStable)
    else
      zedStable;

  yesplaymusicPkg =
    if pkgs.stdenv.hostPlatform.system == "x86_64-linux" then
      pkgs.callPackage ../../pkgs/yesplaymusic { }
    else
      null;
in
{
  options.mcb.personalPackages = {
    enableZed = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install Zed editor in this user's Home Manager profile.";
    };

    zedChannel = lib.mkOption {
      type = lib.types.enum [
        "official-stable"
        "unstable"
        "stable"
      ];
      default = "official-stable";
      description = "Zed package channel for this user: official-stable / unstable / stable.";
    };

    enableYesPlayMusic = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Install YesPlayMusic in this user's Home Manager profile.";
    };
  };

  config = {
    assertions = [
      {
        assertion = (!cfg.enableZed) || (zedPkg != null);
        message = "mcb.personalPackages.enableZed=true but selected mcb.personalPackages.zedChannel is unavailable on this platform.";
      }
      {
        assertion = (!cfg.enableYesPlayMusic) || (yesplaymusicPkg != null);
        message = "mcb.personalPackages.enableYesPlayMusic=true but YesPlayMusic is unavailable on this platform.";
      }
    ];

    home.packages =
      lib.optionals (cfg.enableZed && zedPkg != null) [ zedPkg ]
      ++ lib.optionals (cfg.enableYesPlayMusic && yesplaymusicPkg != null) [ yesplaymusicPkg ];
  };
}
