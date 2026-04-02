# Home Manager Shell 配置入口（fish/aliases/提示符等）。

{ ... }:

{
  programs.fish = {
    enable = true;
  };

  programs.fzf = {
    enable = true;
    enableFishIntegration = true;
    defaultCommand = "fd --type f --hidden --follow --exclude .git";
    defaultOptions = [
      "--height 40%"
      "--layout=reverse"
      "--border=rounded"
      "--preview-window=right:60%"
    ];
    fileWidgetCommand = "fd --type f --hidden --follow --exclude .git";
    fileWidgetOptions = [ "--preview 'bat --color=always {}'" ];
    changeDirWidgetCommand = "fd --type d --hidden --follow --exclude .git";
    changeDirWidgetOptions = [ "--preview 'eza -la --icons {}'" ];
    colors = {
      "bg+" = "#313244";
      bg = "#1e1e2e";
      spinner = "#f5e0dc";
      hl = "#f38ba8";
      fg = "#cdd6f4";
      header = "#f38ba8";
      info = "#cba6f7";
      pointer = "#f5e0dc";
      marker = "#f5e0dc";
      "fg+" = "#cdd6f4";
      prompt = "#cba6f7";
      "hl+" = "#f38ba8";
    };
  };

  programs.atuin = {
    enable = true;
    enableFishIntegration = true;
    settings = {
      auto_sync = true;
      update_check = false;
      search_mode = "fuzzy";
      sync_frequency = "5m";
    };
  };

  programs.direnv = {
    # 自动加载 .envrc
    enable = true;
  };

  programs.zoxide = {
    # 更智能的 cd
    enable = true;
    enableFishIntegration = true;
  };

  programs.yazi = {
    enable = true;
    enableFishIntegration = true;
    shellWrapperName = "yy";
  };

  programs.tmux = {
    # 终端多路复用
    enable = true;
  };

  programs.starship = {
    # 统一提示符主题（配置在 home/users/*/config/starship.toml）
    enable = true;
    enableFishIntegration = true;
  };
}
