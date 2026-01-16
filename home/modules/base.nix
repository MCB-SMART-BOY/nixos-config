{ config, ... }:

let
  homeDir = config.home.homeDirectory;
  heavy = config.mcb.packages.enableHeavyBuilds;
  editor = if heavy then "hx" else "nvim";
  manpager = if heavy then "sh -c 'col -bx | bat -l man -p'" else "less -R";
  terminal = if heavy then "alacritty" else "foot";
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
    "${homeDir}/.cargo/bin"
    "${homeDir}/go/bin"
    "${homeDir}/.local/bin"
  ];
}
