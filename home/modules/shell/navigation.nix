# Shell 导航层：模糊查找、目录跳转和文件管理集成。

{ ... }:

{
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
}
