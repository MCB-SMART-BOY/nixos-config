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
    initContent = builtins.readFile ../config/zsh/.zshrc;
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
    extraConfig = builtins.readFile ../config/tmux/tmux.conf;
  };

  programs.starship = {
    enable = true;
    enableZshIntegration = false;
  };

  xdg.configFile."starship.toml".source = ../config/starship/starship.toml;
}
