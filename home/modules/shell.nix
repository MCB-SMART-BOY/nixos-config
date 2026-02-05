# Home Manager Shell 配置入口（zsh/aliases/提示符等）。

{ config, ... }:

{
  programs.zsh = {
    enable = true;
    # 保持旧的 dotDir，避免 Home Manager 警告
    dotDir = config.home.homeDirectory;
    enableCompletion = true;
    autosuggestion.enable = true;
    syntaxHighlighting.enable = true;
    oh-my-zsh = {
      enable = true;
      # 常用插件集合
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
    # 自动加载 .envrc
    enable = true;
    enableZshIntegration = false;
  };

  programs.zoxide = {
    # 更智能的 cd
    enable = true;
    enableZshIntegration = false;
  };

  programs.tmux = {
    # 终端多路复用
    enable = true;
  };

  programs.starship = {
    # 统一提示符主题（配置在 home/users/*/config/starship.toml）
    enable = true;
    enableZshIntegration = false;
  };
}
