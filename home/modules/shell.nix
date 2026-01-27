{ config, ... }:

{
  programs.zsh = {
    enable = true;
    # Keep legacy dotDir to silence Home Manager warning.
    dotDir = config.home.homeDirectory;
    enableCompletion = true;
    autosuggestion.enable = true;
    syntaxHighlighting.enable = true;
    oh-my-zsh = {
      enable = true;
      plugins = [
        "git"
        "sudo"
        "docker"
        "rust"
        "fzf"
      ];
      theme = "robbyrussell";
    };
  };


  programs.direnv = {
    enable = true;
    enableZshIntegration = false;
  };

  programs.zoxide = {
    enable = true;
    enableZshIntegration = false;
  };

  programs.tmux = {
    enable = true;
  };

  programs.starship = {
    enable = true;
    enableZshIntegration = false;
  };
}
