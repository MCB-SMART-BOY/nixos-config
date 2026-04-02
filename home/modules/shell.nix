# Home Manager Shell 配置入口（fish/aliases/提示符等）。

{ config, ... }:

{
  programs.fish = {
    enable = true;
  };

  programs.direnv = {
    # 自动加载 .envrc
    enable = true;
  };

  programs.zoxide = {
    # 更智能的 cd
    enable = true;
  };

  programs.tmux = {
    # 终端多路复用
    enable = true;
  };

  programs.starship = {
    # 统一提示符主题（配置在 home/users/*/config/starship.toml）
    enable = true;
  };
}
