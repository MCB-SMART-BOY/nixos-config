{ config, ... }:

let
  homeDir = config.home.homeDirectory;
  editor = "hx";
  manpager = "sh -c 'col -bx | bat -l man -p'";
  terminal = "alacritty";
in
{
  home.sessionVariables = {
    EDITOR = editor;
    VISUAL = editor;
    TERMINAL = terminal;
    BROWSER = "firefox";
    MANPAGER = manpager;

    XDG_CONFIG_HOME = "${homeDir}/.config";
    XDG_DATA_HOME = "${homeDir}/.local/share";
    XDG_CACHE_HOME = "${homeDir}/.cache";
    XDG_STATE_HOME = "${homeDir}/.local/state";

    RUSTUP_HOME = "${homeDir}/.rustup";
    CARGO_HOME = "${homeDir}/.cargo";
    GOPATH = "${homeDir}/go";
  };

  home.sessionPath = [
    "/run/wrappers/bin"
    "${homeDir}/.cargo/bin"
    "${homeDir}/go/bin"
    "${homeDir}/.local/bin"
  ];
}
