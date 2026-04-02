# mcbnixos 的 Noctalia 个性化栏入口。
# 具体栏位/按钮定义放在 config/noctalia/settings.nix，便于和其他用户配置目录对齐。

{ config, lib, osConfig ? { }, ... }:

let
  scriptBin = "${config.home.homeDirectory}/.local/bin";
  userNoctaliaSettings = import ./config/noctalia/settings.nix {
    inherit scriptBin osConfig lib;
  };
in
{
  mcb.noctalia.barProfile = "none";

  programs.noctalia-shell.settings = userNoctaliaSettings;
}
