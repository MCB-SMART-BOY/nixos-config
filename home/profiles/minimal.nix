{ ... }:

{
  imports = [
    ../modules/base.nix
    ../modules/shell.nix
    ../modules/git.nix
  ];

  home.sessionVariables = {
    EDITOR = "vi";
    VISUAL = "vi";
    TERMINAL = "bash";
  };
}
