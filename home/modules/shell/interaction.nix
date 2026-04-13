# Shell 交互层：从仓库内恢复的 fish 配置读取交互体验。

{ config, lib, ... }:

let
  fishDir = ../../users + "/${config.home.username}/config/fish/conf.d";
  readFish = name:
    let
      path = fishDir + "/${name}";
    in
    if builtins.pathExists path then builtins.readFile path else "";
  joinFish = names: lib.concatStringsSep "\n\n" (map readFish names);
in
{
  programs.fish = {
    shellInit = lib.mkAfter (readFish "00-env.fish");

    interactiveShellInit = lib.mkAfter (
      joinFish [
        "10-interactive.fish"
        "20-modern-tools.fish"
        "30-shortcuts.fish"
        "40-core-functions.fish"
        "41-navigation-functions.fish"
        "42-nixos-functions.fish"
        "43-file-functions.fish"
      ]
    );

    loginShellInit = lib.mkAfter (readFish "90-startup.fish");
  };
}
