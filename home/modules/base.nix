# Home Manager 基础环境：XDG 目录、PATH、通用环境变量。
# 新手提示：这里影响所有用户的“基础环境变量”。

{ config, lib, ... }:

let
  homeDir = config.home.homeDirectory;
  # 统一编辑器/终端默认值，应用会读取这些变量
  editor = "hx";
  manpager = "less -R";
  terminal = "alacritty";
in
{
  home.sessionVariables = {
    # 常用应用默认值
    EDITOR = lib.mkDefault editor;
    VISUAL = lib.mkDefault editor;
    TERMINAL = lib.mkDefault terminal;
    BROWSER = lib.mkDefault "firefox";
    MANPAGER = lib.mkDefault manpager;

    # XDG 目录：统一用户配置/缓存/数据位置
    XDG_CONFIG_HOME = "${homeDir}/.config";
    XDG_DATA_HOME = "${homeDir}/.local/share";
    XDG_CACHE_HOME = "${homeDir}/.cache";
    XDG_STATE_HOME = "${homeDir}/.local/state";

    # 开发工具目录
    RUSTUP_HOME = "${homeDir}/.rustup";
    CARGO_HOME = "${homeDir}/.cargo";
    GOPATH = "${homeDir}/go";
  };

  # 将常用路径加入 PATH
  home.sessionPath = [
    "/run/wrappers/bin"
    "${homeDir}/.cargo/bin"
    "${homeDir}/go/bin"
    "${homeDir}/.local/bin"
  ];
}
